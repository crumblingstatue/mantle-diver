use serde::{Deserialize, Serialize};

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
