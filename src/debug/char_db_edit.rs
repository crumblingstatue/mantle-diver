use {
    egui_sfml::egui,
    mdv_data::char::{CharDb, Offset},
    mdv_math::types::ScreenVec,
};

#[derive(Default)]
pub struct CharDbEdit {
    pub open: bool,
    pub name_buf: String,
}
impl CharDbEdit {
    pub fn ui(&mut self, ctx: &egui::Context, char_db: &mut CharDb) {
        egui::Window::new("Chardb")
            .open(&mut self.open)
            .show(ctx, |ui| {
                ui.text_edit_singleline(&mut self.name_buf);
                if ui.button("Add").clicked() {
                    char_db.graphic_offsets.insert(
                        self.name_buf.clone(),
                        Offset {
                            left: ScreenVec::default(),
                            right: ScreenVec::default(),
                        },
                    );
                }
                ui.separator();
                char_db.graphic_offsets.retain(|k, v| {
                    let mut retain = true;
                    ui.horizontal(|ui| {
                        ui.label(k);
                        ui.label("l");
                        ui.add(egui::DragValue::new(&mut v.left.x));
                        ui.add(egui::DragValue::new(&mut v.left.y));
                        ui.label("r");
                        ui.add(egui::DragValue::new(&mut v.right.x));
                        ui.add(egui::DragValue::new(&mut v.right.y));
                        if ui.button("x").clicked() {
                            retain = false;
                        }
                    });
                    retain
                })
            });
    }
}
