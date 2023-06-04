use crate::{
    graphics::ScreenVec,
    inventory::{
        ItemDb, ItemDef, TileLayer,
        UseAction::{self, MineTile},
    },
    math::IntRect,
    tiles::{BgTileId, FgTileId, MidTileId},
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
                        use_action: MineTile {
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
                egui::ComboBox::new("action_combo", "Action")
                    .selected_text(def.use_action.text())
                    .show_ui(ui, |ui| {
                        let v = UseAction::MineTile {
                            power: 1.,
                            delay: 1,
                        };
                        let text = v.text();
                        ui.selectable_value(&mut def.use_action, v, text);
                        let v = UseAction::PlaceBgTile { id: BgTileId::DIRT };
                        let text = v.text();
                        ui.selectable_value(&mut def.use_action, v, text);
                        let v = UseAction::PlaceMidTile {
                            id: MidTileId::DIRT,
                        };
                        let text = v.text();
                        ui.selectable_value(&mut def.use_action, v, text);
                        let v = UseAction::PlaceFgTile {
                            id: FgTileId::GRASS,
                        };
                        let text = v.text();
                        ui.selectable_value(&mut def.use_action, v, text);
                        let v = UseAction::RemoveTile {
                            layer: TileLayer::Bg,
                        };
                        let text = v.text();
                        ui.selectable_value(&mut def.use_action, v, text);
                    });
                match &mut def.use_action {
                    UseAction::PlaceBgTile { id } => {
                        ui.label("Tile to place");
                        ui.add(egui::DragValue::new(&mut id.0));
                    }
                    UseAction::PlaceMidTile { id } => {
                        ui.label("Tile to place");
                        ui.add(egui::DragValue::new(&mut id.0));
                    }
                    UseAction::PlaceFgTile { id } => {
                        ui.label("Tile to place");
                        ui.add(egui::DragValue::new(&mut id.0));
                    }
                    UseAction::RemoveTile { layer } => {
                        ui.label("Layer to remove");
                        ui.selectable_value(layer, TileLayer::Bg, "Bg");
                        ui.selectable_value(layer, TileLayer::Mid, "Mid");
                        ui.selectable_value(layer, TileLayer::Fg, "Fg");
                    }
                    MineTile { power, delay } => {
                        ui.label("Power");
                        ui.add(egui::DragValue::new(power));
                        ui.label("Delay");
                        ui.add(egui::DragValue::new(delay));
                    }
                }
            });
    }
}

impl UseAction {
    fn text(&self) -> &'static str {
        match self {
            UseAction::PlaceBgTile { .. } => "Place bg tile",
            UseAction::PlaceMidTile { .. } => "Place mid tile",
            UseAction::PlaceFgTile { .. } => "Place fg tile",
            UseAction::RemoveTile { .. } => "Remove tile",
            MineTile { .. } => "Mine tile",
        }
    }
}
