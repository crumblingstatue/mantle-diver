use {
    crate::data,
    mdv_data::item::{ItemId, ItemStack},
    serde::{Deserialize, Serialize},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Inventory {
    pub slots: Vec<ItemStack>,
}
impl Inventory {
    /// A new inventory filled with some debug items
    pub(crate) fn new_debug() -> Inventory {
        let mut this = Self {
            slots: vec![
                ItemStack {
                    id: data::item::WOOD_PICK,
                    qty: 1,
                },
                ItemStack {
                    id: ItemId::EMPTY,
                    qty: 0,
                },
                ItemStack {
                    id: ItemId::EMPTY,
                    qty: 0,
                },
                ItemStack {
                    id: ItemId::EMPTY,
                    qty: 0,
                },
                ItemStack {
                    id: ItemId::EMPTY,
                    qty: 0,
                },
                ItemStack {
                    id: ItemId::EMPTY,
                    qty: 0,
                },
                ItemStack {
                    id: data::item::TORCH,
                    qty: 100,
                },
                ItemStack {
                    id: data::item::PLATFORM,
                    qty: 9999,
                },
                ItemStack {
                    id: data::item::STONE_WALL,
                    qty: 100,
                },
                ItemStack {
                    id: data::item::DEV_PICK,
                    qty: 1,
                },
            ],
        };
        for _ in 0..40 {
            this.slots.push(ItemStack {
                id: ItemId::EMPTY,
                qty: 0,
            });
        }
        this
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

    /// Returns whether an item could be added into the inventory
    ///
    /// TODO: Add quantity handling
    pub fn item_can_be_added(&self, id: ItemId, _qty: u16) -> bool {
        // First, try to merge with existing slots
        for slot in &self.slots {
            if slot.id == id {
                return true;
            }
        }
        // Next, try to find an empty slot
        for slot in &self.slots {
            if slot.id == ItemId::EMPTY {
                return true;
            }
        }
        false
    }

    pub(crate) fn take_from_slot(&mut self, idx: usize, qty: u16) -> Option<ItemStack> {
        match self.slots.get_mut(idx) {
            Some(slot) => {
                if slot.id == ItemId::EMPTY {
                    None
                } else {
                    match slot.qty.checked_sub(qty) {
                        Some(result) => {
                            slot.qty = result;
                            let id = slot.id;
                            if slot.qty == 0 {
                                slot.id = ItemId::EMPTY;
                            }
                            Some(ItemStack { id, qty })
                        }
                        None => None,
                    }
                }
            }
            None => None,
        }
    }
}
