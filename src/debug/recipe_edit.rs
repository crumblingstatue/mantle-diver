use {
    super::item_db_edit::ItemDbEdit,
    crate::{
        crafting::{Recipe, RecipeDb, Station},
        item::{ItemDb, ItemIdExt},
    },
    mdv_data::item::{ItemId, ItemStack},
};

#[derive(Default)]
pub struct RecdbEd {
    pub open: bool,
    sel_dest: Option<ItemSelectDest>,
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
        egui::Window::new("Recipe editor")
            .open(&mut self.open)
            .show(ctx, |ui| {
                if ui.button("Add new default").clicked() {
                    recipes.recipes.push(Recipe {
                        stations: vec![Station::Player],
                        input: Vec::new(),
                        output: ItemStack {
                            id: ItemId::TORCH,
                            qty: 1,
                        },
                    });
                }
                ui.separator();
                let mut rec_idx = 0;
                recipes.recipes.retain_mut(|rec| {
                    let mut retain = true;
                    ui.label("Station");
                    for station in &mut rec.stations {
                        ui.label(format!("{station:?}"));
                    }
                    ui.horizontal(|ui| {
                        ui.label("Input");
                        if ui.button("✚").clicked() {
                            rec.input.push(ItemStack {
                                id: ItemId::TORCH,
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
                            if ui.button("🗑 Delete recipe").clicked() {
                                retain = false;
                            }
                        });
                    } else {
                        ui.label("Couldn't get item id from item db");
                    }
                    ui.separator();
                    rec_idx += 1;
                    retain
                });
            });
    }
}
