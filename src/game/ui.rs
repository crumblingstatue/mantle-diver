pub struct UiState {
    pub selected_inv_slot: usize,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            selected_inv_slot: 0,
        }
    }
}
