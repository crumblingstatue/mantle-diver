use {
    crate::{
        game::GameState,
        player::{MoveExtra, MovingEnt},
    },
    egui_sfml::egui,
};

#[derive(Default)]
pub struct EntityList {
    pub open: bool,
}

impl EntityList {
    pub fn ui(&mut self, egui_ctx: &egui::Context, game: &mut GameState) {
        egui::Window::new("Entity list")
            .open(&mut self.open)
            .show(egui_ctx, |ui| {
                for en in game.ecw.iter() {
                    ui.label(format!("{:?}", en.entity()));
                    if ui.button("move control").clicked() {
                        game.controlled_en = en.entity();
                        if !en.has::<MoveExtra>() {
                            game.ecw
                                .insert_one(en.entity(), MoveExtra::default())
                                .unwrap();
                            return;
                        }
                    }
                    if let Some(mut mov) = en.get::<&mut MovingEnt>() {
                        ui.add(egui::DragValue::new(&mut mov.vspeed).prefix("vspeed"));
                        ui.add(egui::DragValue::new(&mut mov.mob.en.bb.x).prefix("bb w"));
                        ui.add(egui::DragValue::new(&mut mov.mob.en.bb.y).prefix("bb h"));
                    }
                }
            });
    }
}
