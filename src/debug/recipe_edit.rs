use crate::{
    crafting::{Recipe, RecipeDb, Station},
    inventory::{ItemDb, ItemId, ItemStack},
};

#[derive(Default)]
pub struct RecdbEd {
    pub open: bool,
}
impl RecdbEd {
    pub(crate) fn ui(&mut self, ctx: &egui::Context, recipes: &mut RecipeDb, itemdb: &ItemDb) {
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
                for rec in &mut recipes.recipes {
                    ui.label("Station");
                    for station in &mut rec.stations {
                        ui.label(format!("{station:?}"));
                    }
                    ui.label("Input");
                    for inp in &mut rec.input {
                        let db_en = itemdb.get(inp.id);
                        ui.label(format!("{db_en:?}x{}", inp.qty));
                    }
                    ui.label("Output");
                    if let Some(en) = itemdb.get(rec.output.id) {
                        ui.label(format!("{}x{}", en.name, rec.output.qty));
                    } else {
                        ui.label("Couldn't get item id from item db");
                    }
                    ui.separator();
                }
            });
    }
}
