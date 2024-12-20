use {
    self::{events::EventBuf, ui::UiState},
    crate::{
        audio::AudioCtx,
        command::CmdVec,
        debug::{DebugState, DBG_OVR},
        graphics::ScreenRes,
        input::Input,
        inventory::Inventory,
        item::ItemDbExt,
        math::{wp_to_tp, WPosSc, WorldPos, TILE_SIZE},
        player::PlayerBundle,
        res::{Res, ResAudio},
        save::Save,
        tiles::TileDbExt,
        time::{daylight, tick_of_day},
        world::{TilePos, World},
    },
    egui_sfml::egui,
    fnv::FnvHashMap,
    mdv_data::{
        char::CharDb,
        item::{ItemDb, ItemStack},
        recipe::RecipeDb,
        tile::{LayerAccess, TileDb},
    },
    mdv_math::types::ScreenVec,
    rand::{thread_rng, Rng},
    std::path::{Path, PathBuf},
};

mod events;
pub mod rendering;
mod systems;
mod ui;

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
    pub spawn_point: WorldPos,
    pub transient_tile_states: TransientTileStates,
    pub last_mine_attempt: u64,
    pub last_tile_place: u64,
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
    pub ui: UiState,
    /// Whether to use smart cursor for targeting blocks
    pub smart_cursor: bool,
    /// Tile pos to highlight (for smart cursor, etc.)
    pub highlight_tp: Option<TilePos>,
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

impl GameState {
    pub(crate) fn new(world_name: String, path: PathBuf, res: &Res) -> GameState {
        let mut spawn_point = WorldPos::SURFACE_CENTER;
        let mut tile_db = TileDb::load_or_default("data");
        tile_db.update_rects(&res.atlas.rects);
        let mut itemdb = ItemDb::load_or_default("data");
        itemdb.update_rects(&res.atlas.rects);
        let mut inventory = Inventory::new_empty();
        let mut world;
        let mut plr = PlayerBundle::new_at(spawn_point);
        spawn_point.y -= WPosSc::try_from(plr.mov.mob.en.bb.y).unwrap();
        plr.mov.mob.en.pos.y = i32::try_from(spawn_point.y).unwrap();
        match Save::load(&path) {
            Ok(save) => {
                inventory = save.inventory;
                world = World::new(&world_name, path, save.world_seed);
                world.ticks = save.world_ticks;
                plr.dat.update_from_save(&save.player);
                plr.health = save.player.health;
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
            spawn_point,
            transient_tile_states: Default::default(),
            last_mine_attempt: 0,
            last_tile_place: 0,
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
            ui: UiState::default(),
            smart_cursor: false,
            highlight_tp: None,
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub(crate) fn run_systems(
        &mut self,
        debug: &DebugState,
        input: &mut Input,
        mouse_wpos: WorldPos,
        mouse_tpos: TilePos,
        rt_size: ScreenRes,
        res: &Res,
        au_ctx: &mut AudioCtx,
        au_res: &ResAudio,
        cmd: &mut CmdVec,
        worlds_dir: &Path,
        scale: u8,
        egui_ctx: &egui::Context,
    ) {
        self.ui.update_rects(&self.inventory, rt_size);
        self.ambient_light = daylight(tick_of_day(self.world.ticks));
        mdv_math::util::min_max_clamp(&mut self.ambient_light, 85, 200);
        if self.respawn_timer > 0 {
            self.respawn_timer -= 1;
        } else if self.player_en == hecs::Entity::DANGLING {
            self.player_en = self.ecw.spawn(PlayerBundle::new_at(self.spawn_point));
            self.controlled_en = self.player_en;
        }
        let hud_msg = systems::ui_hud_input_system(self, input, scale, rt_size);
        if self.ui.menu.open {
            systems::pause_menu::pause_menu_system(self, input, cmd, worlds_dir, au_ctx);
            return;
        }
        systems::craft_ui_system(self, egui_ctx, ScreenRes::from_sf_vec(res.atlas.tex.size()));
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
        systems::move_system(self, rt_size.to_vec(), debug);
        if !hud_msg.cursor_occupied() {
            systems::interact_system(self, input, mouse_tpos, mouse_wpos);
            systems::item_use_system(self, input, mouse_tpos, au_res, au_ctx, mouse_wpos, debug);
        }
        systems::biome_watch_system(self, au_ctx, res);
        systems::inventory_input_system(self, input);
        systems::item_drop_claim_system(self, au_ctx, au_res);
        systems::transient_blocks_system(self);
        systems::health_system(self);
        self.world.ticks += 1;
        let ev_buf = std::mem::take(&mut self.event_buf);
        events::process_events(self, ev_buf, au_ctx, au_res);
    }

    pub(crate) fn selected_item_is(&self, id: mdv_data::item::ItemId) -> bool {
        self.selected_slot().map_or(false, |slot| slot.id == id)
    }

    pub(crate) fn selected_slot(&self) -> Option<&ItemStack> {
        self.inventory.slots.get(self.ui.selected_inv_slot)
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
