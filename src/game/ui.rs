use {
    super::systems::pause_menu::Menu,
    crate::graphics::ScreenRes,
    mdv_math::{
        types::{ScreenRect, ScreenSc},
        util::center_offset,
    },
};

#[derive(Default)]
pub struct UiState {
    pub selected_inv_slot: usize,
    pub persistent_selected_inv_slot: usize,
    pub menu: Menu,
    pub inv: Inventory,
    pub craft: CraftWnd,
    pub hotbar_rects: [ScreenRect; 10],
    pub inv_rects: Vec<ScreenRect>,
}

impl UiState {
    #[expect(clippy::cast_possible_wrap, reason = "Doesn't exceed i16::MAX")]
    pub fn update_rects(&mut self, inv: &super::Inventory, rt_res: ScreenRes) {
        for i in 0..10u16 {
            let (x, y) = ((i * 44) + 8, (rt_res.h - 48));
            let rect = &mut self.hotbar_rects[i as usize];
            rect.x = x as i16;
            rect.y = y as i16;
            rect.w = 36;
            rect.h = 36;
        }
        let inv_rect = Inventory::screen_rect(rt_res, inv.slots.len());
        let mut y_off = 24;
        let mut x_off = 8;
        self.inv_rects.clear();
        for i in 0..inv.slots.len() {
            self.inv_rects.push(ScreenRect {
                x: inv_rect.x + x_off,
                y: inv_rect.y + y_off,
                w: 36,
                h: 36,
            });
            x_off += 42;
            if (i + 1) % 10 == 0 {
                x_off = 8;
                y_off += 42;
            }
        }
    }

    pub(crate) fn select_inv_slot(&mut self, slot: usize) {
        self.selected_inv_slot = slot;
        self.persistent_selected_inv_slot = slot;
    }
}

#[derive(Default)]
pub struct Inventory {
    pub open: bool,
}

const ITEMS_PER_ROW: u8 = 10;

impl Inventory {
    #[expect(clippy::cast_possible_wrap, reason = "resolution is always positive")]
    pub fn screen_rect(res: ScreenRes, slots: usize) -> ScreenRect {
        let items_per_row = ScreenSc::from(ITEMS_PER_ROW);
        let rows = ScreenSc::try_from(slots).unwrap() / items_per_row;
        let w = (items_per_row * 40) + 34;
        let h = (rows * 40) + 34;
        let x = center_offset(w, res.w as i16);
        let y = center_offset(h, res.h as i16);
        ScreenRect { x, y, w, h }
    }
}

#[derive(Default)]
pub struct CraftWnd {
    pub open: bool,
    pub selected_recipe: Option<usize>,
}
