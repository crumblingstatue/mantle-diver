use {
    super::item_db_edit::ItemDbEdit,
    crate::data,
    egui_sfml::egui,
    mdv_data::{
        item::{ItemDb, ItemStack},
        recipe::{Recipe, RecipeDb, Station},
    },
};

#[derive(Default)]
pub struct RecdbEd {
    pub open: bool,
    sel_dest: Option<ItemSelectDest>,
    pub active_idx: usize,
}

#[derive(PartialEq, Eq)]
enum ItemSelectDest {
    Input { recipe: usize, input: usize },
    Output { recipe: usize },
}

impl RecdbEd {
    pub(crate) fn ui(
        &mut self,
        ctx: &egui::Context,
        recipes: &mut RecipeDb,
        itemdb: &ItemDb,
        idb_ed: &mut ItemDbEdit,
    ) {
        let mut open = self.open;
        egui::Window::new("Recipe editor")
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.set_max_width(100.0);
                        self.list_ui(ui, recipes, itemdb);
                    });
                    ui.vertical(|ui| {
                        self.recipe_ui(ui, recipes, itemdb, self.active_idx, idb_ed);
                    });
                });
            });
        self.open = open;
    }

    fn list_ui(&mut self, ui: &mut egui::Ui, recipes: &mut RecipeDb, itemdb: &ItemDb) {
        if ui.button("Add new default").clicked() {
            recipes.recipes.push(Recipe {
                stations: vec![Station::Player],
                input: Vec::new(),
                output: ItemStack {
                    id: data::item::TORCH,
                    qty: 1,
                },
            });
        }
        ui.separator();
        for (i, rec) in recipes.recipes.iter().enumerate() {
            let name = itemdb
                .get(rec.output.id)
                .map_or("<Unknown>", |def| &def.name);
            let amount = rec.output.qty;
            if ui
                .selectable_label(self.active_idx == i, format!("{name} x {amount}"))
                .clicked()
            {
                self.active_idx = i;
            }
        }
    }

    fn recipe_ui(
        &mut self,
        ui: &mut egui::Ui,
        recipes: &mut RecipeDb,
        itemdb: &ItemDb,
        rec_idx: usize,
        idb_ed: &mut ItemDbEdit,
    ) {
        let rec = &mut recipes.recipes[rec_idx];
        ui.label("Station");
        for station in &mut rec.stations {
            ui.label(format!("{station:?}"));
        }
        ui.horizontal(|ui| {
            ui.label("Input");
            if ui.button("✚").clicked() {
                rec.input.push(ItemStack {
                    id: data::item::TORCH,
                    qty: 1,
                });
            }
        });
        let mut input_idx = 0;
        rec.input.retain_mut(|inp| {
            let mut retain = true;
            let db_en = itemdb.get(inp.id).unwrap();
            ui.horizontal(|ui| {
                if ui.button(&db_en.name).clicked() {
                    idb_ed.open = true;
                    idb_ed.extern_sel_mode = true;
                    self.sel_dest = Some(ItemSelectDest::Input {
                        recipe: rec_idx,
                        input: input_idx,
                    });
                }
                if self.sel_dest
                    == Some(ItemSelectDest::Input {
                        recipe: rec_idx,
                        input: input_idx,
                    })
                {
                    if let Some(id) = idb_ed.sel_for_extern {
                        inp.id = id;
                    }
                }
                ui.label("x");
                ui.add(egui::DragValue::new(&mut inp.qty));
                if ui.button("➖").clicked() {
                    retain = false;
                }
            });
            input_idx += 1;
            retain
        });
        ui.label("Output");
        if let Some(en) = itemdb.get(rec.output.id) {
            ui.horizontal(|ui| {
                if ui.button(&en.name).clicked() {
                    idb_ed.open = true;
                    idb_ed.extern_sel_mode = true;
                    self.sel_dest = Some(ItemSelectDest::Output { recipe: rec_idx });
                }
                if self.sel_dest == Some(ItemSelectDest::Output { recipe: rec_idx }) {
                    if let Some(id) = idb_ed.sel_for_extern {
                        rec.output.id = id;
                    }
                }
                ui.label("x");
                ui.add(egui::DragValue::new(&mut rec.output.qty));
            });
        } else {
            ui.label("Couldn't get item id from item db");
        }
        if ui.button("🗑 Remove").clicked() {
            recipes.recipes.remove(rec_idx);
        }
    }
}
