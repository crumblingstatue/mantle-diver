use {
    crate::{
        app::{SoundPlayer, TileColEn},
        graphics::{ScreenSc, ScreenVec},
        input::Input,
        inventory::{Inventory, ItemDb},
        itemdrop::Itemdrop,
        math::{step_towards, wp_to_tp, WorldPos, TILE_SIZE},
        res::{Res, ResAudio},
        save::Save,
        tiles::{TileDb, TileDef, TileId, TileLayer},
        world::{TilePos, World},
    },
    derivative::Derivative,
    egui_inspect::{derive::Inspect, Inspect},
    fnv::FnvHashMap,
    rand::{thread_rng, Rng},
    sfml::{
        system::{Vector2f, Vector2u},
        window::Key,
    },
    std::{ops::Index, path::PathBuf},
};

mod rendering;

#[derive(Derivative, Inspect)]
#[derivative(Debug)]
pub struct GameState {
    pub camera_offset: WorldPos,
    pub world: World,
    pub gravity: f32,
    pub current_biome: Biome,
    pub prev_biome: Biome,
    #[derivative(Debug = "ignore")]
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

#[derive(Debug, Inspect)]
pub struct TransientBlockState {
    /// If block health reaches 0, it gets destroyed
    pub health: f32,
    pub rot: f32,
    pub scale: f32,
}

#[derive(Debug, Inspect)]
pub struct LightSource {
    pub pos: ScreenVec,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Inspect)]
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
    pub fn update<'a>(
        &mut self,
        input: &Input,
        snd: &mut SoundPlayer<'a>,
        aud: &'a ResAudio,
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

    pub(crate) fn new(world_name: String, path: PathBuf, res: &Res, res_path: &str) -> GameState {
        let mut spawn_point = WorldPos::SURFACE_CENTER;
        spawn_point.y -= 1104;
        let mut tile_db = TileDb::load_or_default(res_path);
        tile_db.update_rects(&res.atlas.rects);
        let mut itemdb = ItemDb::load_or_default(res_path);
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
}

fn process_tile_item_drop<L: TileLayer>(
    tile_db: &TileDb,
    item_drops: &mut Vec<Itemdrop>,
    id: TileId<L>,
    pos: &TilePos,
) where
    TileDb: Index<TileId<L>, Output = TileDef<L>>,
    L::SpecificDef: Inspect,
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
