use {
    super::DebugState,
    crate::{cmdline::CmdLine, command::CmdVec},
    egui_sfml::egui::{self, TextBuffer},
    std::fmt::Write,
};

#[derive(Default, Debug)]
pub struct Console {
    pub show: bool,
    pub cmdline: String,
    pub log: String,
    pub just_opened: bool,
    pub history: Vec<String>,
}

pub fn console_ui(ctx: &egui::Context, debug: &mut DebugState, cmd: &mut CmdVec) {
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
                    crate::cmdline::Dispatch::ToggleWorldMgr => debug.world_mgr.toggle(),
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
