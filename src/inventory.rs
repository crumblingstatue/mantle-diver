use {
    mdv_data::item::{ItemDb, ItemId, ItemStack},
    serde::{Deserialize, Serialize},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Inventory {
    pub slots: Vec<ItemStack>,
    pub grabbed: Option<ItemStack>,
}
impl Inventory {
    /// A new inventory filled with empty slots
    pub(crate) fn new_empty() -> Self {
        let mut slots = Vec::new();
        for _ in 0..20 {
            slots.push(ItemStack {
                id: ItemId::EMPTY,
                qty: 0,
            });
        }
        Self {
            slots,
            grabbed: None,
        }
    }
    /// Returns false if the item can't be added (full inv)
    pub fn add(&mut self, id: ItemId, qty: u16, itemdb: &ItemDb) -> bool {
        let stackable = itemdb.get(id).map_or(false, |en| en.stackable);
        if !stackable {
            // Just try to find an empty slot
            for slot in &mut self.slots {
                if slot.id == ItemId::EMPTY {
                    slot.id = id;
                    // Quantity doesn't matter for non-stackable items
                    slot.qty = 1;
                    return true;
                }
            }
        }
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
            // Make any slots that reached a qty of 0 empty
            if slot.qty == 0 {
                slot.id = ItemId::EMPTY;
            }
            if qty == 0 {
                break;
            }
        }
    }

    pub(crate) fn find_item_idx(&self, id: ItemId) -> Option<usize> {
        for (i, slot) in self.slots.iter().enumerate() {
            if slot.id == id && slot.qty > 0 {
                return Some(i);
            }
        }
        None
    }
}
