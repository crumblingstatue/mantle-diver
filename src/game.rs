use {
    self::systems::Menu,
    crate::{
        app::SoundPlayer,
        char::CharDb,
        command::CmdVec,
        debug::{DebugState, DBG_OVR},
        graphics::ScreenVec,
        input::Input,
        inventory::{Inventory, ItemDb},
        math::{wp_to_tp, WorldPos, TILE_SIZE},
        player::PlayerBundle,
        res::{Res, ResAudio},
        save::Save,
        tiles::TileDb,
        world::{TilePos, World},
    },
    fnv::FnvHashMap,
    rand::{thread_rng, Rng},
    sfml::system::{Vector2f, Vector2u},
    std::path::{Path, PathBuf},
};

pub mod rendering;
mod systems;

pub struct GameState {
    /// Camera offset.
    /// It points to top left corner on the screen.
    pub camera_offset: WorldPos,
    pub world: World,
    pub gravity: f32,
    pub current_biome: Biome,
    pub prev_biome: Biome,
    pub ambient_light: u8,
    pub light_sources: Vec<LightSource>,
    pub tile_db: TileDb,
    pub char_db: CharDb,
    pub inventory: Inventory,
    pub itemdb: ItemDb,
    pub selected_inv_slot: usize,
    pub spawn_point: WorldPos,
    pub transient_block_state: FnvHashMap<TilePos, TransientBlockState>,
    pub last_mine_attempt: u64,
    pub last_tile_place: u64,
    pub menu: Menu,
    pub ecw: hecs::World,
    pub ecb: hecs::CommandBuffer,
    pub player_en: hecs::Entity,
    pub controlled_en: hecs::Entity,
    pub paused: bool,
    pub pause_next_frame: bool,
}

#[derive(Debug)]
pub struct TransientBlockState {
    /// If block health reaches 0, it gets destroyed
    pub health: f32,
    pub rot: f32,
    pub scale: f32,
}

#[derive(Debug)]
pub struct LightSource {
    pub pos: ScreenVec,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Biome {
    Surface,
    Underground,
}

trait SfVec2fExt {
    fn scv_off(&self, off: ScreenVec) -> Self;
}

impl SfVec2fExt for Vector2f {
    fn scv_off(&self, off: ScreenVec) -> Self {
        Self {
            x: self.x + f32::from(off.x),
            y: self.y + f32::from(off.y),
        }
    }
}

impl GameState {
    pub(crate) fn new(world_name: String, path: PathBuf, res: &Res) -> GameState {
        let mut spawn_point = WorldPos::SURFACE_CENTER;
        spawn_point.y -= 1104;
        let mut tile_db = TileDb::load_or_default("data");
        tile_db.update_rects(&res.atlas.rects);
        let mut itemdb = ItemDb::load_or_default("data");
        itemdb.update_rects(&res.atlas.rects);
        let mut inventory = Inventory::new_debug();
        let world;
        let mut plr = PlayerBundle::new_at(spawn_point);
        match Save::load(&path) {
            Ok(save) => {
                inventory = save.inventory;
                world = World::new(&world_name, path, save.world_seed);
                plr.dat.update_from_save(&save.player);
            }
            Err(e) => {
                log::error!("Failed to load save: {e}");
                world = World::new(&world_name, path, thread_rng().gen());
            }
        }
        let mut ecw = hecs::World::new();
        let player_en = ecw.spawn(plr);
        log::info!("=== Spawned Player ===");
        Self {
            camera_offset: spawn_point,
            world,
            gravity: 0.55,
            current_biome: Biome::Surface,
            prev_biome: Biome::Surface,
            ambient_light: 0,
            light_sources: Vec::new(),
            tile_db,
            inventory,
            itemdb,
            selected_inv_slot: 0,
            spawn_point,
            transient_block_state: Default::default(),
            last_mine_attempt: 0,
            last_tile_place: 0,
            menu: Menu::default(),
            char_db: CharDb::load().unwrap(),
            ecw,
            ecb: hecs::CommandBuffer::default(),
            player_en,
            controlled_en: player_en,
            paused: false,
            pause_next_frame: false,
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub(crate) fn run_systems(
        &mut self,
        debug: &DebugState,
        input: &mut Input,
        mouse_tpos: TilePos,
        rt_size: Vector2u,
        music_sink: &mut rodio::Sink,
        res: &Res,
        snd: &mut SoundPlayer,
        aud: &ResAudio,
        cmd: &mut CmdVec,
        worlds_dir: &Path,
    ) {
        systems::general_input_system(self, input);
        if self.menu.open {
            systems::pause_menu_system(self, input, cmd, worlds_dir);
            return;
        }
        if self.paused {
            return;
        }
        if self.pause_next_frame {
            self.paused = true;
            self.pause_next_frame = false;
            DBG_OVR.clear();
        }
        if debug.freecam {
            systems::freecam_move_system(self, input);
        } else {
            systems::move_control_system(self, input);
        }
        systems::move_system(self, ScreenVec::from_sf_resolution(rt_size), debug);
        systems::item_use_system(self, input, mouse_tpos, aud, snd);
        systems::biome_watch_system(self, music_sink, res);
        systems::inventory_input_system(self, input);
        systems::item_drop_claim_system(self, snd, aud);
        systems::transient_blocks_system(self);
        self.world.ticks += 1;
    }
}

pub fn for_each_tile_on_screen(
    camera_offset: WorldPos,
    rt_size: ScreenVec,
    mut f: impl FnMut(TilePos, ScreenVec),
) {
    let ts_i16 = i16::from(TILE_SIZE);
    let ts_us = usize::from(TILE_SIZE);
    for y in (-ts_i16..rt_size.y + ts_i16).step_by(ts_us) {
        for x in (-ts_i16..rt_size.x + ts_i16).step_by(ts_us) {
            let cam_mod = camera_offset.tile_modulo();
            f(
                TilePos {
                    x: wp_to_tp(camera_offset.x.saturating_add(x.try_into().unwrap_or(0))),
                    y: wp_to_tp(camera_offset.y.saturating_add(y.try_into().unwrap_or(0))),
                },
                ScreenVec {
                    x: x - cam_mod.x,
                    y: y - cam_mod.y,
                },
            )
        }
    }
}
