use {
    crate::{
        command::{Cmd, CmdVec},
        game::GameState,
        save,
    },
    egui_sfml::egui,
    rand::{thread_rng, Rng},
    std::path::{Path, PathBuf},
};

#[derive(Default, Debug)]
pub struct WorldManager {
    open: bool,
    just_opened: bool,
    world_dirs: Vec<PathBuf>,
    new_world_name: String,
}
impl WorldManager {
    pub fn ui(
        &mut self,
        ctx: &egui::Context,
        game: &GameState,
        worlds_path: &Path,
        cmd: &mut CmdVec,
    ) {
        let mut open = self.open;
        egui::Window::new("World manager")
            .open(&mut open)
            .show(ctx, |ui| {
                if self.just_opened {
                    self.world_dirs = save::world_dirs(worlds_path);
                    self.just_opened = false;
                }
                ui.label(&format!("Current world: {}", game.world.name));
                ui.separator();
                let mut refresh = false;
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for dir in &self.world_dirs {
                            ui.label(dir.display().to_string());
                            ui.horizontal(|ui| {
                                if ui.button("Play").clicked() {
                                    cmd.push(Cmd::LoadWorld(
                                        dir.file_name().unwrap().to_str().unwrap().to_string(),
                                    ));
                                }
                                if ui.button("Delete").clicked() {
                                    std::fs::remove_dir_all(dir).unwrap();
                                    refresh = true;
                                }
                            });
                            ui.separator();
                        }
                    });
                ui.separator();
                if ui.button("Delete all").clicked() {
                    for dir in &self.world_dirs {
                        std::fs::remove_dir_all(dir).unwrap();
                    }
                    refresh = true;
                }
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.new_world_name)
                            .hint_text("New world name"),
                    );
                    if ui
                        .add_enabled(!self.new_world_name.is_empty(), egui::Button::new("Create"))
                        .clicked()
                    {
                        cmd.push(Cmd::LoadWorld(self.new_world_name.clone()));
                        refresh = true;
                    }
                    if ui.button("Create random").clicked() {
                        let n: u32 = thread_rng().gen();
                        cmd.push(Cmd::LoadWorld(n.to_string()));
                        refresh = true;
                    }
                });

                if refresh {
                    self.world_dirs = save::world_dirs(worlds_path);
                }
            });
        self.open = open;
    }

    pub(crate) fn toggle(&mut self) {
        self.open ^= true;
        self.just_opened = true;
    }
}
