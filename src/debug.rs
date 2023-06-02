use {
    crate::{
        char::Offset,
        cmdline::CmdLine,
        command::{Cmd, CmdVec},
        game::GameState,
        graphics::ScreenVec,
        math::{px_per_frame_to_km_h, WorldPos},
        res::Res,
        stringfmt::LengthDisp,
        tiles::tiledb_edit_ui::TileDbEdit,
    },
    egui::TextBuffer,
    gamedebug_core::IMMEDIATE,
    rand::{thread_rng, Rng},
    sfml::graphics::Color,
    std::{
        fmt::Write,
        path::{Path, PathBuf},
    },
};

#[derive(Default)]
pub struct DebugState {
    pub panel: bool,
    pub freecam: bool,
    pub tiledb_edit: TileDbEdit,
    pub show_atlas: bool,
    pub console: Console,
    pub player_bb: bool,
    pub chardb_edit: CharDbEdit,
    world_mgr: WorldManager,
}

#[derive(Default)]
pub struct CharDbEdit {
    pub open: bool,
    pub name_buf: String,
}
impl CharDbEdit {
    fn ui(&mut self, ctx: &egui::Context, char_db: &mut crate::char::CharDb) {
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
                        ui.label("r");
                        ui.add(egui::DragValue::new(&mut v.right.x));
                        if ui.button("x").clicked() {
                            retain = false;
                        }
                    });
                    retain
                })
            });
    }
}

#[derive(Default, Debug)]
pub struct Console {
    pub show: bool,
    pub cmdline: String,
    pub log: String,
    pub just_opened: bool,
    pub history: Vec<String>,
}

fn debug_panel_ui(debug: &mut DebugState, game: &mut GameState, ctx: &egui::Context) {
    let mut open = debug.panel;
    egui::Window::new("Debug (F12)")
        .open(&mut open)
        .show(ctx, |ui| {
            if debug.freecam {
                ui.label("Cam x");
                ui.add(egui::DragValue::new(&mut game.camera_offset.x));
                ui.label("Cam y");
                ui.add(egui::DragValue::new(&mut game.camera_offset.y));
                let co = game.camera_offset;
                ui.label(format!(
                    "Cam Depth: {}",
                    LengthDisp(co.y as f32 - WorldPos::SURFACE as f32)
                ));
                ui.label(format!(
                    "Cam offset from center: {}",
                    LengthDisp(co.x as f32 - WorldPos::CENTER as f32)
                ));
            } else {
                ui.label(format!("Player Depth: {}", game.world.player.depth_disp()));
                ui.label(format!(
                    "Player offset from center: {}",
                    LengthDisp(game.world.player.col_en.en.pos.x as f32 - WorldPos::CENTER as f32)
                ));
                ui.label(format!(
                    "Hspeed: {} ({} km/h)",
                    game.world.player.hspeed,
                    px_per_frame_to_km_h(game.world.player.hspeed)
                ));
                ui.label(format!(
                    "Vspeed: {} ({} km/h)",
                    game.world.player.vspeed,
                    px_per_frame_to_km_h(game.world.player.vspeed)
                ));
            }
            ui.collapsing("Player", |ui| {
                ui.label("Skin color");
                color_edit_button(ui, &mut game.world.player.skin_color);
                ui.label("Eye color");
                color_edit_button(ui, &mut game.world.player.eye_color);
                ui.label("Hair color");
                color_edit_button(ui, &mut game.world.player.hair_color);
                ui.label("Shirt color");
                color_edit_button(ui, &mut game.world.player.shirt_color);
                ui.label("Pants color");
                color_edit_button(ui, &mut game.world.player.pants_color);
                ui.label("Shoes color");
                color_edit_button(ui, &mut game.world.player.shoes_color);
                ui.checkbox(&mut debug.player_bb, "Draw bb");
            });
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                IMMEDIATE.for_each(|msg| {
                    ui.label(msg);
                });
            });
            IMMEDIATE.clear();
        });
    debug.panel = open;
}

fn color_edit_button(ui: &mut egui::Ui, c: &mut Color) {
    let mut rgb = [c.r, c.g, c.b];
    ui.color_edit_button_srgb(&mut rgb);
    c.r = rgb[0];
    c.g = rgb[1];
    c.b = rgb[2];
}

pub(crate) fn do_debug_ui(
    ctx: &egui::Context,
    debug: &mut DebugState,
    game: &mut GameState,
    res: &mut Res,
    cmd: &mut CmdVec,
    worlds_path: &Path,
) {
    debug_panel_ui(debug, game, ctx);
    debug.tiledb_edit.ui(
        ctx,
        &mut game.tile_db,
        &game.itemdb,
        res.atlas.tex.size(),
        cmd,
    );
    debug.chardb_edit.ui(ctx, &mut game.char_db);
    console_ui(ctx, debug, cmd);
    debug.world_mgr.ui(ctx, game, worlds_path, cmd);
}

fn console_ui(ctx: &egui::Context, debug: &mut DebugState, cmd: &mut CmdVec) {
    let mut open = debug.console.show;
    egui::Window::new("Console (F11)")
        .open(&mut open)
        .show(ctx, |ui| {
            let up_arrow =
                ui.input_mut(|inp| inp.consume_key(egui::Modifiers::default(), egui::Key::ArrowUp));
            let re =
                ui.add(egui::TextEdit::singleline(&mut debug.console.cmdline).hint_text("Command"));
            if debug.console.just_opened {
                re.request_focus();
            }
            if re.lost_focus() && ui.input(|inp| inp.key_pressed(egui::Key::Enter)) {
                re.request_focus();
                let cmdline = match CmdLine::parse_cmdline(&debug.console.cmdline) {
                    Ok(cmd) => cmd,
                    Err(e) => {
                        writeln!(&mut debug.console.log, "{e}").unwrap();
                        debug.console.history.push(debug.console.cmdline.take());
                        return;
                    }
                };
                debug.console.history.push(debug.console.cmdline.take());
                match cmdline.dispatch(debug) {
                    crate::cmdline::Dispatch::Cmd(command) => cmd.push(command),
                    crate::cmdline::Dispatch::ClearConsole => debug.console.log.clear(),
                    crate::cmdline::Dispatch::ToggleAtlas => debug.show_atlas ^= true,
                    crate::cmdline::Dispatch::ToggleWorldMgr => {
                        debug.world_mgr.open ^= true;
                        debug.world_mgr.just_opened = true;
                    }
                    crate::cmdline::Dispatch::Noop => {}
                }
            }
            if up_arrow {
                if let Some(line) = debug.console.history.pop() {
                    debug.console.cmdline = line;
                }
            }
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.add(egui::TextEdit::multiline(&mut &debug.console.log[..]));
                });
        });
    debug.console.just_opened = false;
    debug.console.show = open;
}

#[derive(Default, Debug)]
struct WorldManager {
    open: bool,
    just_opened: bool,
    world_dirs: Vec<PathBuf>,
    new_world_name: String,
}
impl WorldManager {
    fn ui(
        &mut self,
        ctx: &egui::Context,
        game: &mut GameState,
        worlds_path: &Path,
        cmd: &mut CmdVec,
    ) {
        let mut open = self.open;
        egui::Window::new("World manager")
            .open(&mut open)
            .show(ctx, |ui| {
                if self.just_opened {
                    self.world_dirs = world_dirs(worlds_path);
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
                    self.world_dirs = world_dirs(worlds_path);
                }
            });
        self.open = open;
    }
}

pub fn world_dirs(worlds_path: &Path) -> Vec<PathBuf> {
    let mut v = Vec::new();
    for child in std::fs::read_dir(worlds_path).unwrap() {
        let child = child.unwrap();
        v.push(child.path());
    }
    v
}
