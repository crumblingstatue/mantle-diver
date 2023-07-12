use super::systems::pause_menu::Menu;

pub struct UiState {
    pub selected_inv_slot: usize,
    pub menu: Menu,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            selected_inv_slot: 0,
            menu: Menu::default(),
        }
    }
}
