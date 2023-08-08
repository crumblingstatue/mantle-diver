use {
    crate::{egui_ext::EguiUiExt, res::Res},
    mdv_math::types::IntRect,
};

#[derive(Default)]
pub struct GraphicPicker {
    pub open: bool,
    /// Should be "consumed" by whatever is querying last picked item
    pub picked: Option<(String, IntRect)>,
}
impl GraphicPicker {
    pub(crate) fn ui(
        &mut self,
        ctx: &egui::Context,
        atlas_size: crate::graphics::ScreenRes,
        res: &Res,
    ) {
        let mut open = self.open;
        egui::Window::new("Graphic picker")
            .open(&mut open)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        let mut w_accum = 0;
                        for (name, rect) in &res.atlas.rects {
                            ui.group(|ui| {
                                ui.label(name);
                                if ui.graphic_image_button(rect, atlas_size).clicked() {
                                    self.picked = Some((name.clone(), *rect));
                                    self.open = false;
                                }
                            });
                            w_accum += rect.w;
                            if w_accum > 200 {
                                ui.end_row();
                                w_accum = 0;
                            }
                        }
                    });
                });
            });
        if !open {
            self.open = false;
        }
    }
}
