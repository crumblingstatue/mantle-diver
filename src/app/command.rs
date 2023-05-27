use {
    super::App,
    crate::{command::Cmd, game::GameState, res::Res, texture_atlas::AtlasBundle},
    std::fmt::Write,
};

pub(super) fn dispatch(app: &mut App, res: &mut Res) {
    for cmd in app.cmdvec.drain(..) {
        match cmd {
            Cmd::QuitApp => app.should_quit = true,
            Cmd::ToggleFreecam => app.debug.freecam ^= true,
            Cmd::TeleportPlayer { pos, relative } => {
                if relative {
                    let s2dc = pos.to_s2dc();
                    app.game.world.player.col_en.en.pos.x += s2dc.x;
                    app.game.world.player.col_en.en.pos.y += s2dc.y;
                } else {
                    app.game.world.player.col_en.en.pos = pos.to_s2dc()
                }
            }
            Cmd::TeleportPlayerSpawn => {
                app.game.world.player.col_en.en.pos = app.game.spawn_point.to_s2dc()
            }
            Cmd::GiveItemByName { name, amount } => {
                for (id, item) in app.game.itemdb.iter() {
                    if item.name == name {
                        app.game.inventory.add(id, amount);
                        return;
                    }
                }
                writeln!(
                    &mut app.debug.console.log,
                    "Item with name '{name}' not found"
                )
                .unwrap();
            }
            Cmd::ToggleTileDbEdit => app.debug.tiledb_edit.open ^= true,
            Cmd::SetScale(scale) => app.scale = scale,
            Cmd::LoadWorld(name) => {
                app.game.world.save();
                let path = app.worlds_dir.join(&name);
                app.game = GameState::new(name, path, res);
            }
            Cmd::ReloadGraphics => {
                res.atlas = AtlasBundle::new(&app.cfg.res_folder_path).unwrap();
                app.game.tile_db.update_rects(&res.atlas.rects);
            }
            Cmd::SetBgTileAtCursor(id) => {
                app.game.world.tile_at_mut(app.last_mouse_tpos).bg = id;
            }
            Cmd::SetMidTileAtCursor(id) => {
                app.game.world.tile_at_mut(app.last_mouse_tpos).mid = id;
            }
            Cmd::SetFgTileAtCursor(id) => {
                app.game.world.tile_at_mut(app.last_mouse_tpos).fg = id;
            }
        }
    }
}
