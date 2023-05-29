use {
    self::systems::Menu,
    crate::{
        app::{SoundPlayer, TileColEn},
        command::CmdVec,
        debug::DebugState,
        graphics::{ScreenSc, ScreenVec},
        input::Input,
        inventory::{Inventory, ItemDb},
        itemdrop::Itemdrop,
        math::{wp_to_tp, WorldPos},
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

mod rendering;
mod systems;

pub struct GameState {
    pub camera_offset: WorldPos,
    pub world: World,
    pub gravity: f32,
    pub current_biome: Biome,
    pub prev_biome: Biome,
    pub ambient_light: u8,
    pub light_sources: Vec<LightSource>,
    pub tile_db: TileDb,
    pub inventory: Inventory,
    pub itemdb: ItemDb,
    pub selected_inv_slot: usize,
    pub spawn_point: WorldPos,
    pub transient_block_state: FnvHashMap<TilePos, TransientBlockState>,
    pub last_mine_attempt: u64,
    pub item_drops: Vec<Itemdrop>,
    pub menu: Menu,
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
            x: self.x + off.x as f32,
            y: self.y + off.y as f32,
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
        let seed;
        match Save::load(&path) {
            Ok(save) => {
                inventory = save.inventory;
                seed = save.world_seed;
            }
            Err(e) => {
                log::error!("Failed to load save: {e}");
                seed = thread_rng().gen();
            }
        }
        Self {
            camera_offset: spawn_point,
            world: World::new(spawn_point, &world_name, path, seed),
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
            item_drops: Default::default(),
            menu: Menu::default(),
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub(crate) fn run_systems(
        &mut self,
        debug: &DebugState,
        input: &mut Input,
        mouse_world_pos: WorldPos,
        mouse_tpos: TilePos,
        rt_size: Vector2u,
        music_sink: &mut rodio::Sink,
        res: &Res,
        snd: &mut SoundPlayer,
        on_screen_tile_ents: &mut Vec<TileColEn>,
        aud: &ResAudio,
        cmd: &mut CmdVec,
        worlds_dir: &Path,
    ) {
        if self.menu.open {
            systems::pause_menu_system(self, input, cmd, worlds_dir);
            return;
        }
        systems::general_input_system(self, input);
        if debug.freecam {
            systems::freecam_move_system(self, mouse_world_pos, input);
        } else {
            systems::player_move_system(self, input, rt_size, on_screen_tile_ents);
        }
        systems::item_use_system(self, input, mouse_tpos, aud, snd);
        systems::biome_watch_system(self, music_sink, res);
        systems::inventory_input_system(self, input);
        systems::item_drop_claim_system(self, on_screen_tile_ents, snd, aud);
        systems::transient_blocks_system(self);
        self.world.ticks += 1;
    }
}

pub fn for_each_tile_on_screen(
    camera_offset: WorldPos,
    rt_size: Vector2u,
    mut f: impl FnMut(TilePos, ScreenVec),
) {
    for y in (-32..(rt_size.y as i16) + 32).step_by(32) {
        for x in (-32..(rt_size.x as i16) + 32).step_by(32) {
            f(
                TilePos {
                    x: wp_to_tp(camera_offset.x.saturating_add(x.try_into().unwrap_or(0))),
                    y: wp_to_tp(camera_offset.y.saturating_add(y.try_into().unwrap_or(0))),
                },
                ScreenVec {
                    x: ((x as i64) - ((camera_offset.x as i64) % 32)) as ScreenSc,
                    y: ((y as i64) - ((camera_offset.y as i64) % 32)) as ScreenSc,
                },
            )
        }
    }
}
