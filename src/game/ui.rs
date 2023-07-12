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
    pub hotbar_rects: [ScreenRect; 10],
    pub inv_rects: Vec<ScreenRect>,
}

impl UiState {
    #[expect(clippy::cast_possible_wrap, reason = "Doesn't exceed i16::MAX")]
    pub fn update_rects(&mut self, rt_res: ScreenRes) {
        for i in 0..10u16 {
            let (x, y) = ((i * 44) + 8, (rt_res.h - 48));
            let rect = &mut self.hotbar_rects[i as usize];
            rect.x = x as i16;
            rect.y = y as i16;
            rect.w = 36;
            rect.h = 36;
        }
    }
}

#[derive(Default)]
pub struct Inventory {
    pub open: bool,
}

impl Inventory {
    #[expect(clippy::cast_possible_wrap, reason = "resolution is always positive")]
    pub fn screen_rect(res: ScreenRes) -> ScreenRect {
        let w = 450;
        let h = 270;
        let x = center_offset(w, res.w as i16);
        let y = center_offset(h, res.h as i16);
        ScreenRect { x, y, w, h }
    }
}
