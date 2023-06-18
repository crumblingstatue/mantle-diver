use {
    crate::{
        math::{IntRect, ScreenVec, TILE_SIZE},
        tiles::{BgTileId, LayerAccess, MidTileId},
    },
    mdv_data::ron_pretty_cfg,
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

/// A stack of items (one or more item of a kind)
#[derive(Debug, Serialize, Deserialize)]
pub struct ItemStack {
    pub id: ItemId,
    pub qty: ItemQty,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemDef {
    pub name: String,
    pub graphic_name: String,
    #[serde(skip)]
    pub tex_rect: IntRect,
    pub draw_off: ScreenVec,
    /// Primary use
    pub use1: UseAction,
    /// Secondary use
    pub use2: UseAction,
    pub consumable: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub enum UseAction {
    #[default]
    Nothing,
    PlaceBgTile {
        id: BgTileId,
    },
    PlaceMidTile {
        id: MidTileId,
    },
    RemoveTile {
        layer: LayerAccess,
    },
    /// Mine mid level block
    MineTile {
        power: f32,
        delay: u64,
    },
    /// Mine background wall
    MineBgTile {
        power: f32,
        delay: u64,
    },
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ItemDb {
    pub db: Vec<ItemDef>,
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
            self.db.get(id.0 as usize - 1)
        }
    }
    pub fn try_save(&self, data_path: &str) {
        match ron::ser::to_string_pretty(self, ron_pretty_cfg()) {
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
                    def.tex_rect.w = rect.w.min(TILE_SIZE.into());
                    def.tex_rect.h = rect.h.min(TILE_SIZE.into());
                    log::info!("Updated rect for {}: {:?}", def.graphic_name.as_str(), rect);
                } else {
                    log::error!("Missing texture for {:?}", def.graphic_name.as_str());
                }
            } else {
                log::warn!("Empty graphic name!");
            }
        }
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "We won't have more than 65535 items"
    )]
    pub fn iter(&self) -> impl Iterator<Item = (ItemId, &ItemDef)> {
        self.db
            .iter()
            .enumerate()
            .map(|(i, def)| (ItemId(i as u16 + 1), def))
    }
}
