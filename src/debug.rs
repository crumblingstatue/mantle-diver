mod char_db_edit;
mod console;
mod world_manager;

use {
    self::{
        char_db_edit::CharDbEdit,
        console::{console_ui, Console},
        world_manager::WorldManager,
    },
    crate::{
        command::CmdVec,
        game::GameState,
        math::{px_per_frame_to_km_h, WorldPos},
        res::Res,
        stringfmt::LengthDisp,
        tiles::tiledb_edit_ui::TileDbEdit,
    },
    gamedebug_core::IMMEDIATE,
    sfml::graphics::Color,
    std::path::Path,
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
