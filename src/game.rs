use {
    crate::{
        app::{SoundPlayer, TileColEn},
        graphics::{ScreenSc, ScreenVec},
        input::Input,
        inventory::{self, Inventory, ItemDb, ItemId, UseAction},
        itemdrop::Itemdrop,
        math::{step_towards, wp_to_tp, WorldPos, TILE_SIZE},
        res::{Res, ResAudio},
        save::Save,
        tiles::{TileDb, TileDef, TileId, TileLayer},
        world::{TilePos, World},
    },
    fnv::FnvHashMap,
    rand::{seq::SliceRandom, thread_rng, Rng},
    sfml::{
        system::{Vector2f, Vector2u},
        window::Key,
    },
    std::{ops::Index, path::PathBuf},
};

mod rendering;

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
    pub fn update(
        &mut self,
        input: &Input,
        snd: &mut SoundPlayer,
        aud: &ResAudio,
        on_screen_tile_ents: &[TileColEn],
    ) {
        if input.pressed(Key::Num1) {
            self.selected_inv_slot = 0;
        }
        if input.pressed(Key::Num2) {
            self.selected_inv_slot = 1;
        }
        if input.pressed(Key::Num3) {
            self.selected_inv_slot = 2;
        }
        if input.pressed(Key::Num4) {
            self.selected_inv_slot = 3;
        }
        if input.pressed(Key::Num5) {
            self.selected_inv_slot = 4;
        }
        if input.pressed(Key::Num6) {
            self.selected_inv_slot = 5;
        }
        if input.pressed(Key::Num7) {
            self.selected_inv_slot = 6;
        }
        if input.pressed(Key::Num8) {
            self.selected_inv_slot = 7;
        }
        if input.pressed(Key::Num9) {
            self.selected_inv_slot = 8;
        }
        if input.pressed(Key::Num0) {
            self.selected_inv_slot = 9;
        }
        // Claim item drops player contacts with
        self.item_drops.retain_mut(|itemdrop| {
            itemdrop.s2dc_en.move_x(itemdrop.hspeed, |en, off| {
                for t_en in on_screen_tile_ents {
                    if en.would_collide(&t_en.col, off) {
                        itemdrop.hspeed = 0.0;
                        return true;
                    }
                }
                false
            });
            step_towards(&mut itemdrop.hspeed, 0.0, 0.03);
            itemdrop.s2dc_en.move_y(itemdrop.vspeed, |en, off| {
                for t_en in on_screen_tile_ents {
                    if en.would_collide(&t_en.col, off) {
                        itemdrop.vspeed = 0.0;
                        return true;
                    }
                }
                false
            });
            itemdrop.vspeed += self.gravity;
            let mut retain = true;
            #[expect(clippy::collapsible_if)]
            if self.world.player.col_en.en.collides(&itemdrop.s2dc_en.en) {
                if self.inventory.add(itemdrop.id, 1) {
                    snd.play(aud, "etc/pickup");
                    retain = false;
                }
            }
            retain
        });
        // Update transient blocks
        self.transient_block_state.retain(|pos, state| {
            step_towards(&mut state.rot, 0.0, 0.9);
            step_towards(&mut state.scale, 1.0, 0.04);
            // Kill tiles with 0 health
            let mut retain = true;
            if state.health <= 0.0 {
                let tile = &mut self.world.tile_at_mut(*pos);
                process_tile_item_drop(&self.tile_db, &mut self.item_drops, tile.mid, pos);
                tile.mid = TileId::EMPTY;
                // If the mid is destroyed, the front content pops off as well
                process_tile_item_drop(&self.tile_db, &mut self.item_drops, tile.fg, pos);
                tile.fg = TileId::EMPTY;
                retain = false;
            }
            retain
        });
        self.world.ticks += 1;
    }

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
        }
    }

    pub(crate) fn item_use_system(
        &mut self,
        input: &Input,
        mouse_tpos: TilePos,
        aud: &ResAudio,
        snd: &mut SoundPlayer,
    ) {
        if !input.lmb_down {
            return;
        }
        let Some(active_slot) = self.inventory.slots.get_mut(self.selected_inv_slot) else {
            log::error!("Selected slot {} out of bounds", self.selected_inv_slot);
            return;
        };
        if active_slot.qty == 0 {
            return;
        }
        let Some(itemdef) = &self.itemdb.get(active_slot.id) else {
            return;
        };
        let ticks = self.world.ticks;
        let t = self.world.tile_at_mut(mouse_tpos);
        match &itemdef.use_action {
            UseAction::PlaceBgTile { id } => {
                if t.bg.empty() {
                    t.bg = *id;
                    active_slot.qty -= 1;
                }
            }
            UseAction::PlaceMidTile { id } => {
                if t.mid.empty() {
                    t.mid = *id;
                    active_slot.qty -= 1;
                }
            }
            UseAction::PlaceFgTile { id } => {
                if t.fg.empty() {
                    t.fg = *id;
                    active_slot.qty -= 1;
                }
            }
            UseAction::RemoveTile { layer } => match layer {
                inventory::TileLayer::Bg => t.bg = TileId::EMPTY,
                inventory::TileLayer::Mid => t.mid = TileId::EMPTY,
                inventory::TileLayer::Fg => t.fg = TileId::EMPTY,
            },
            UseAction::MineTile { power, delay } => 'block: {
                if t.mid == TileId::EMPTY || ticks - self.last_mine_attempt < *delay {
                    break 'block;
                }
                let tdef = &self.tile_db[t.mid];
                let state =
                    self.transient_block_state
                        .entry(mouse_tpos)
                        .or_insert(TransientBlockState {
                            health: tdef.health,
                            rot: 0.0,
                            scale: 1.0,
                        });
                let mut rng = thread_rng();
                let abs_rot = rng.gen_range(8.0..=16.0);
                let max_scale = rng.gen_range(1.1..=1.3);
                let min_scale = rng.gen_range(0.8..=0.9);
                state.rot = *[-abs_rot, abs_rot].choose(&mut rng).unwrap();
                state.scale = *[min_scale, max_scale].choose(&mut rng).unwrap();
                state.health -= power;
                if let Some(hit_snd) = &tdef.hit_sound {
                    snd.play(aud, hit_snd);
                }
                self.last_mine_attempt = ticks;
            }
        }
        // Make sure that fully consumed stacks are cleared
        if active_slot.qty == 0 {
            active_slot.id = ItemId::EMPTY;
        }
    }
}

fn process_tile_item_drop<L: TileLayer>(
    tile_db: &TileDb,
    item_drops: &mut Vec<Itemdrop>,
    id: TileId<L>,
    pos: &TilePos,
) where
    TileDb: Index<TileId<L>, Output = TileDef<L>>,
{
    if id.empty() {
        log::warn!("Empty tile id: {id:?}");
        return;
    }
    let Some(drop) = &tile_db[id].item_drop else {
        return
    };
    let amount = thread_rng().gen_range(drop.qty_range.clone());
    for _ in 0..amount {
        item_drops.push(Itemdrop::new_at(
            drop.id,
            WorldPos {
                x: pos.x * TILE_SIZE as u32 + TILE_SIZE as u32 / 2,
                y: pos.y * TILE_SIZE as u32 + TILE_SIZE as u32 / 2,
            },
        ));
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
