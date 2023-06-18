use {
    self::{events::EventBuf, systems::pause_menu::Menu},
    crate::{
        audio::AudioCtx,
        command::CmdVec,
        debug::{DebugState, DBG_OVR},
        input::Input,
        inventory::Inventory,
        item::ItemDb,
        math::{wp_to_tp, ScreenVecExt, WPosSc, WorldPos, TILE_SIZE},
        player::PlayerBundle,
        res::{Res, ResAudio},
        save::Save,
        tiles::TileDbExt,
        time::{daylight, tick_of_day},
        world::{TilePos, World},
    },
    fnv::FnvHashMap,
    mdv_data::{
        char::CharDb,
        recipe::RecipeDb,
        tile::{LayerAccess, TileDb},
    },
    mdv_math::types::ScreenVec,
    rand::{thread_rng, Rng},
    sfml::system::{Vector2f, Vector2u},
    std::path::{Path, PathBuf},
};

mod events;
pub mod rendering;
mod systems;

#[derive(Hash, PartialEq, Eq)]
pub struct TilestateKey {
    pos: TilePos,
    layer: LayerAccess,
}

type TransientTileStates = FnvHashMap<TilestateKey, TransientTileState>;

pub struct GameState {
    /// Camera offset.
    /// It points to top left corner on the screen.
    pub camera_offset: WorldPos,
    pub world: World,
    pub gravity: f32,
    pub current_biome: Biome,
    pub prev_biome: Biome,
    pub ambient_light: u8,
    pub tile_db: TileDb,
    pub char_db: CharDb,
    pub recipe_db: RecipeDb,
    pub inventory: Inventory,
    pub itemdb: ItemDb,
    pub selected_inv_slot: usize,
    pub spawn_point: WorldPos,
    pub transient_tile_states: TransientTileStates,
    pub last_mine_attempt: u64,
    pub last_tile_place: u64,
    pub menu: Menu,
    pub ecw: hecs::World,
    pub ecb: hecs::CommandBuffer,
    pub player_en: hecs::Entity,
    pub controlled_en: hecs::Entity,
    pub paused: bool,
    pub pause_next_frame: bool,
    pub event_buf: EventBuf,
    // Respawn timer for player
    pub respawn_timer: u32,
    pub tile_interact_radius: u16,
    pub item_pickup_radius: u16,
}

#[derive(Debug)]
pub struct TransientTileState {
    /// If block health reaches 0, it gets destroyed
    pub health: f32,
    pub rot: f32,
    pub scale: f32,
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
        let mut tile_db = TileDb::load_or_default("data");
        tile_db.update_rects(&res.atlas.rects);
        let mut itemdb = ItemDb::load_or_default("data");
        itemdb.update_rects(&res.atlas.rects);
        let mut inventory = Inventory::new_debug();
        let world;
        let mut plr = PlayerBundle::new_at(spawn_point);
        spawn_point.y -= WPosSc::try_from(plr.mov.mob.en.bb.y).unwrap();
        plr.mov.mob.en.pos.y = i32::try_from(spawn_point.y).unwrap();
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
            ambient_light: 128,
            tile_db,
            inventory,
            itemdb,
            selected_inv_slot: 0,
            spawn_point,
            transient_tile_states: Default::default(),
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
            event_buf: Default::default(),
            respawn_timer: 0,
            tile_interact_radius: 113,
            item_pickup_radius: 80,
            recipe_db: RecipeDb::load_or_default("data"),
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub(crate) fn run_systems(
        &mut self,
        debug: &DebugState,
        input: &mut Input,
        mouse_wpos: WorldPos,
        mouse_tpos: TilePos,
        rt_size: Vector2u,
        res: &Res,
        au_ctx: &mut AudioCtx,
        au_res: &ResAudio,
        cmd: &mut CmdVec,
        worlds_dir: &Path,
    ) {
        self.ambient_light = daylight(tick_of_day(self.world.ticks));
        if self.respawn_timer > 0 {
            self.respawn_timer -= 1;
        } else if self.player_en == hecs::Entity::DANGLING {
            self.player_en = self.ecw.spawn(PlayerBundle::new_at(self.spawn_point));
            self.controlled_en = self.player_en;
        }
        systems::general_input_system(self, input);
        if self.menu.open {
            systems::pause_menu::pause_menu_system(self, input, cmd, worlds_dir);
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
        systems::item_use_system(self, input, mouse_tpos, au_res, au_ctx, mouse_wpos);
        systems::biome_watch_system(self, au_ctx, res);
        systems::inventory_input_system(self, input);
        systems::item_drop_claim_system(self, au_ctx, au_res);
        systems::transient_blocks_system(self);
        systems::health_system(self);
        self.world.ticks += 1;
        let ev_buf = std::mem::take(&mut self.event_buf);
        events::process_events(self, ev_buf, au_ctx, au_res);
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
