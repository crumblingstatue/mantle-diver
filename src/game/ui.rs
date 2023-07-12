use super::systems::pause_menu::Menu;

#[derive(Default)]
pub struct UiState {
    pub selected_inv_slot: usize,
    pub menu: Menu,
    pub inv: Inventory,
}

#[derive(Default)]
pub struct Inventory {
    pub open: bool,
}
