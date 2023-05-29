use {
    crate::{
        graphics::ScreenVec,
        math::{IntRect, TILE_SIZE},
        tiles::{BgTileId, FgTileId, MidTileId},
    },
    ron::ser::PrettyConfig,
    serde::{Deserialize, Serialize},
};

/// We won't have more than 65535 different items
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemId(pub u16);

impl ItemId {
    pub const EMPTY: Self = Self(0);
    //pub const DIRT_BLOCK: ItemId = Self(1);
    pub const TORCH: ItemId = Self(2);
    pub const PLATFORM: ItemId = Self(3);
    pub const WOOD_PICK: ItemId = Self(4);
    //pub const PANZERIUM: ItemId = Self(5);
    pub const STONE_WALL: ItemId = Self(6);
    //pub const STONE_BLOCK: ItemId = Self(7);
    //pub const COAL: ItemId = Self(8);
    pub const DEV_PICK: ItemId = Self(9);
}

/// We won't have more than 65535 item quantity in a single slot
pub type ItemQty = u16;

/// Inventory slot
#[derive(Debug, Serialize, Deserialize)]
pub struct Slot {
    pub id: ItemId,
    pub qty: ItemQty,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Inventory {
    pub slots: Vec<Slot>,
}
impl Inventory {
    /// A new inventory filled with some debug items
    pub(crate) fn new_debug() -> Inventory {
        Self {
            slots: vec![
                Slot {
                    id: ItemId::WOOD_PICK,
                    qty: 1,
                },
                Slot {
                    id: ItemId::EMPTY,
                    qty: 0,
                },
                Slot {
                    id: ItemId::EMPTY,
                    qty: 0,
                },
                Slot {
                    id: ItemId::EMPTY,
                    qty: 0,
                },
                Slot {
                    id: ItemId::EMPTY,
                    qty: 0,
                },
                Slot {
                    id: ItemId::EMPTY,
                    qty: 0,
                },
                Slot {
                    id: ItemId::TORCH,
                    qty: 100,
                },
                Slot {
                    id: ItemId::PLATFORM,
                    qty: 9999,
                },
                Slot {
                    id: ItemId::STONE_WALL,
                    qty: 100,
                },
                Slot {
                    id: ItemId::DEV_PICK,
                    qty: 1,
                },
            ],
        }
    }
    /// Returns false if the item can't be added (full inv)
    pub fn add(&mut self, id: ItemId, qty: u16) -> bool {
        // First, try to merge with existing slots
        for slot in &mut self.slots {
            if slot.id == id {
                slot.qty += qty;
                return true;
            }
        }
        // Next, try to find an empty slot
        for slot in &mut self.slots {
            if slot.id == ItemId::EMPTY {
                slot.id = id;
                slot.qty = qty;
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemDef {
    pub name: String,
    pub graphic_name: String,
    #[serde(skip)]
    pub tex_rect: IntRect,
    pub draw_off: ScreenVec,
    pub use_action: UseAction,
    pub consumable: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum TileLayer {
    Bg,
    Mid,
    Fg,
}

#[derive(Debug, Serialize, Deserialize)]
#[expect(clippy::enum_variant_names)]
pub enum UseAction {
    PlaceBgTile { id: BgTileId },
    PlaceMidTile { id: MidTileId },
    PlaceFgTile { id: FgTileId },
    RemoveTile { layer: TileLayer },
    MineTile { power: f32, delay: u64 },
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ItemDb {
    db: Vec<ItemDef>,
}

const PATH: &str = "items.ron";

impl ItemDb {
    pub fn load_or_default(data_path: &str) -> Self {
        match std::fs::read_to_string(format!("{data_path}/{PATH}")) {
            Ok(data) => match ron::from_str(&data) {
                Ok(db) => db,
                Err(e) => {
                    log::warn!("Failed to load item database: {e}\nCreating default.");
                    Default::default()
                }
            },
            Err(e) => {
                log::warn!("Failed to load item database: {e}\nCreating default.");
                Default::default()
            }
        }
    }
    pub fn get(&self, id: ItemId) -> Option<&ItemDef> {
        if id == ItemId::EMPTY {
            None
        } else {
            Some(&self.db[id.0 as usize - 1])
        }
    }
    pub fn try_save(&self, data_path: &str) {
        match ron::ser::to_string_pretty(self, PrettyConfig::default()) {
            Ok(str) => match std::fs::write(format!("{data_path}/{PATH}"), str) {
                Ok(()) => {}
                Err(e) => log::warn!("Failed to save item db: {e}"),
            },
            Err(e) => log::warn!("Failed to save item db: {e}"),
        }
    }

    pub(crate) fn update_rects(&mut self, rects: &std::collections::HashMap<String, IntRect>) {
        for def in &mut self.db {
            if !def.graphic_name.is_empty() {
                if let Some(rect) = rects.get(def.graphic_name.as_str()) {
                    def.tex_rect = *rect;
                    def.tex_rect.w = rect.w.min(TILE_SIZE as _);
                    def.tex_rect.h = rect.h.min(TILE_SIZE as _);
                    log::info!("Updated rect for {}: {:?}", def.graphic_name.as_str(), rect);
                } else {
                    log::error!("Missing texture for {:?}", def.graphic_name.as_str());
                }
            } else {
                log::warn!("Empty graphic name!");
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (ItemId, &ItemDef)> {
        self.db
            .iter()
            .enumerate()
            .map(|(i, def)| (ItemId(i as u16 + 1), def))
    }
}
