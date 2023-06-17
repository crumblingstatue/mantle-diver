use crate::{
    graphics::ScreenVec,
    inventory::{
        ItemDb, ItemDef, ItemId, TileLayer,
        UseAction::{self},
    },
    math::IntRect,
    tiles::{BgTileId, MidTileId},
};

#[derive(Default)]
pub struct ItemDbEdit {
    pub open: bool,
    sel_idx: usize,
    /// Other editors can open this editor to ask for an item selection
    pub extern_sel_mode: bool,
    /// Selected item for external item requests
    pub sel_for_extern: Option<ItemId>,
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
                        use1: UseAction::Nothing,
                        use2: UseAction::Nothing,
                        consumable: false,
                    })
                }
                ui.separator();
                for (i, def) in itemdb.db.iter().enumerate() {
                    ui.horizontal(|ui| {
                        if ui.selectable_label(i == self.sel_idx, &def.name).clicked() {
                            self.sel_idx = i;
                        }
                        if self.extern_sel_mode {
                            #[expect(
                                clippy::cast_possible_truncation,
                                reason = "We won't have more than 65535 items"
                            )]
                            if ui.button("Select this").clicked() {
                                self.sel_for_extern = Some(ItemId((i + 1) as u16));
                                self.extern_sel_mode = false;
                            }
                        }
                    });
                }
                ui.separator();
                let Some(def) = itemdb.db.get_mut(self.sel_idx) else {
                    ui.label("No item selected (or out of bounds)");
                    return
                };
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
                use_dropdown_combo(&mut def.use1, ui, "Primary use");
                use_dropdown_combo(&mut def.use2, ui, "Secondary use");
            });
    }
}

fn use_dropdown_combo(use_field: &mut UseAction, ui: &mut egui::Ui, label: &str) {
    egui::ComboBox::new(label, label)
        .selected_text(use_field.text())
        .show_ui(ui, |ui| {
            let v = UseAction::Nothing;
            let text = v.text();
            ui.selectable_value(use_field, v, text);
            let v = UseAction::MineTile {
                power: 1.,
                delay: 1,
            };
            let text = v.text();
            ui.selectable_value(use_field, v, text);
            let v = UseAction::MineBgTile {
                power: 1.,
                delay: 1,
            };
            let text = v.text();
            ui.selectable_value(use_field, v, text);
            let v = UseAction::PlaceBgTile { id: BgTileId::DIRT };
            let text = v.text();
            ui.selectable_value(use_field, v, text);
            let v = UseAction::PlaceMidTile {
                id: MidTileId::DIRT,
            };
            let text = v.text();
            ui.selectable_value(use_field, v, text);
            let v = UseAction::RemoveTile {
                layer: TileLayer::Bg,
            };
            let text = v.text();
            ui.selectable_value(use_field, v, text);
        });
    match use_field {
        UseAction::PlaceBgTile { id } => {
            ui.label("Tile to place");
            ui.add(egui::DragValue::new(&mut id.0));
        }
        UseAction::PlaceMidTile { id } => {
            ui.label("Tile to place");
            ui.add(egui::DragValue::new(&mut id.0));
        }
        UseAction::RemoveTile { layer } => {
            ui.label("Layer to remove");
            ui.selectable_value(layer, TileLayer::Bg, "Bg");
            ui.selectable_value(layer, TileLayer::Mid, "Mid");
        }
        UseAction::MineTile { power, delay } => {
            ui.label("Power");
            ui.add(egui::DragValue::new(power));
            ui.label("Delay");
            ui.add(egui::DragValue::new(delay));
        }
        UseAction::MineBgTile { power, delay } => {
            ui.label("Power");
            ui.add(egui::DragValue::new(power));
            ui.label("Delay");
            ui.add(egui::DragValue::new(delay));
        }
        UseAction::Nothing => {}
    }
}

impl UseAction {
    fn text(&self) -> &'static str {
        match self {
            UseAction::PlaceBgTile { .. } => "Place bg tile",
            UseAction::PlaceMidTile { .. } => "Place mid tile",
            UseAction::RemoveTile { .. } => "Remove tile",
            UseAction::MineTile { .. } => "Mine tile",
            UseAction::MineBgTile { .. } => "Mine bg tile",
            UseAction::Nothing => "Nothing",
        }
    }
}
