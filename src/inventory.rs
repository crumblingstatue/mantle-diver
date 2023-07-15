use {
    crate::data,
    mdv_data::item::{ItemId, ItemStack},
    serde::{Deserialize, Serialize},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Inventory {
    pub slots: Vec<ItemStack>,
    pub grabbed: Option<ItemStack>,
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
                    id: ItemId::EMPTY,
                    qty: 0,
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
            grabbed: None,
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
    /// How many items of type `id` do we have?
    pub(crate) fn count_item(&self, id: ItemId) -> u64 {
        let mut sum = 0;
        for slot in &self.slots {
            if slot.id == id {
                sum += u64::from(slot.qty);
            }
        }
        sum
    }
    /// Removes `qty` amount of item `id`, or all of it if there is not enough.
    pub(crate) fn remove(&mut self, id: ItemId, mut qty: u16) {
        for slot in &mut self.slots {
            if slot.id == id {
                if slot.qty < qty {
                    qty -= slot.qty;
                    slot.qty = 0;
                } else {
                    slot.qty -= qty;
                    qty = 0;
                }
            }
            if qty == 0 {
                break;
            }
        }
    }
}
