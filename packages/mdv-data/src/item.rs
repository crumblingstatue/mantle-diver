use {
    crate::{
        ron_pretty_cfg,
        tile::{BgTileId, LayerAccess, MidTileId},
    },
    mdv_math::types::{IntRect, ScreenVec},
    serde::{Deserialize, Serialize},
};

/// We won't have more than 65535 different items
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemId(pub u16);

impl ItemId {
    pub const EMPTY: Self = Self(0);
}

/// We won't have more than 65535 item quantity in a single slot
pub type ItemQty = u16;

/// A stack of items (one or more item of a kind)
#[derive(Debug, Serialize, Deserialize)]
pub struct ItemStack {
    pub id: ItemId,
    pub qty: ItemQty,
}

impl ItemStack {
    // TODO: Not an associated constant because of a rust-analyzer bug that crashes RA.
    //
    // Bug should be reported to RA.
    pub const fn empty() -> Self {
        Self {
            id: ItemId::EMPTY,
            qty: 0,
        }
    }
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
    #[serde(alias = "consumable")]
    pub stackable: bool,
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
