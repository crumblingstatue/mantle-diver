use {
    super::systems::pause_menu::Menu,
    crate::graphics::ScreenRes,
    mdv_math::{types::ScreenRect, util::center_offset},
};

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

impl Inventory {
    #[expect(clippy::cast_possible_wrap, reason = "resolution is always positive")]
    pub fn screen_rect(res: ScreenRes) -> ScreenRect {
        let w = 800;
        let h = 400;
        let x = center_offset(w, res.w as i16);
        let y = center_offset(h, res.h as i16);
        ScreenRect { x, y, w, h }
    }
}
