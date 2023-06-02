use crate::{
    graphics::ScreenVec,
    inventory::{ItemDb, ItemDef},
    math::IntRect,
};

#[derive(Default)]
pub struct ItemDbEdit {
    pub open: bool,
    sel_idx: usize,
}

impl ItemDbEdit {
    pub fn ui(&mut self, ctx: &egui::Context, itemdb: &mut ItemDb) {
        egui::Window::new("Item db")
            .open(&mut self.open)
            .show(ctx, |ui| {
                if ui.button("New item").clicked() {
                    itemdb.db.push(ItemDef {
                        name: "New item".into(),
                        graphic_name: "".into(),
                        tex_rect: IntRect::default(),
                        draw_off: ScreenVec::default(),
                        use_action: crate::inventory::UseAction::MineTile {
                            power: 0.0,
                            delay: 0,
                        },
                        consumable: false,
                    })
                }
                ui.separator();
                for (i, def) in itemdb.db.iter().enumerate() {
                    if ui.selectable_label(i == self.sel_idx, &def.name).clicked() {
                        self.sel_idx = i;
                    }
                }
                ui.separator();
                let def = &mut itemdb.db[self.sel_idx];
                ui.horizontal(|ui| {
                    ui.label("Name");
                    ui.text_edit_singleline(&mut def.name);
                });
                ui.horizontal(|ui| {
                    ui.label("Graphic name");
                    ui.text_edit_singleline(&mut def.graphic_name);
                });
                ui.horizontal(|ui| {
                    ui.label("Graphic offset");
                    ui.add(egui::DragValue::new(&mut def.draw_off.x));
                    ui.add(egui::DragValue::new(&mut def.draw_off.y));
                });
                ui.checkbox(&mut def.consumable, "Consumable");
            });
    }
}
