use {
    crate::{
        audio::AudioCtx,
        command::{Cmd, CmdVec},
        game::GameState,
        input::{Input, InputAction},
        save::world_dirs,
    },
    rand::{thread_rng, Rng},
    sfml::{graphics::Color, window::Key},
    std::path::Path,
};

pub type MenuStack = Vec<MenuList>;
pub type MenuList = Vec<MenuItem>;
pub struct MenuItem {
    pub text: String,
    action: MenuAction,
}

enum MenuAction {
    NewRandom,
    Load,
    LoadWorld(String),
    Settings,
    Quit,
    Back,
    Input,
    Rebind(InputAction),
    MusicVolume,
    SfxVolume,
    DesktopFullscreen,
}

pub fn pause_menu_system(
    game: &mut GameState,
    input: &mut Input,
    cmd: &mut CmdVec,
    worlds_dir: &Path,
    aud: &AudioCtx,
) {
    if let Some(act) = game.ui.menu.action_to_rebind {
        game.ui.menu.sel_color = Color::RED;
        if let Some(key) = input.just_pressed_raw {
            input.key_bindings.insert(act, key);
            game.ui.menu.action_to_rebind = None;
            if let Some(items) = game.ui.menu.stack.last_mut() {
                *items = build_keyconfig_menu(input);
            }
        }
        return;
    }
    game.ui.menu.sel_color = Color::YELLOW;
    let enter = input.pressed_raw(Key::Enter);
    let left = input.pressed_raw(Key::Left);
    let right = input.pressed_raw(Key::Right);
    if let Some(list) = game.ui.menu.stack.last_mut() {
        let current_menu_item = &mut list[game.ui.menu.cursor];
        match &mut current_menu_item.action {
            MenuAction::NewRandom => {
                if enter {
                    let n: u32 = thread_rng().gen();
                    cmd.push(Cmd::LoadWorld(n.to_string()));
                }
            }
            MenuAction::Load => 'block: {
                if !enter {
                    break 'block;
                }
                let mut list = Vec::new();
                for dir in world_dirs(worlds_dir) {
                    let Some(last) = dir.file_name() else {
                        log::error!("World doesn't have file name component");
                        continue;
                    };
                    let last = last.to_string_lossy().to_string();
                    list.push(MenuItem {
                        text: last.clone(),
                        action: MenuAction::LoadWorld(last),
                    })
                }
                list.push(MenuItem {
                    text: "Back".into(),
                    action: MenuAction::Back,
                });
                game.ui.menu.stack.push(list);
                game.ui.menu.cursor = 0;
            }
            MenuAction::Quit => {
                if enter {
                    cmd.push(Cmd::QuitApp);
                }
            }
            MenuAction::LoadWorld(name) => {
                if enter {
                    cmd.push(Cmd::LoadWorld(name.clone()))
                }
            }
            MenuAction::Back => {
                if enter {
                    game.ui.menu.cursor = 0;
                    game.ui.menu.stack.pop();
                    if game.ui.menu.stack.is_empty() {
                        game.ui.menu.open = false;
                    }
                }
            }
            MenuAction::Settings => {
                if enter {
                    let items = vec![
                        MenuItem {
                            text: "Input".into(),
                            action: MenuAction::Input,
                        },
                        MenuItem {
                            text: vol_text("Music", aud.mus_vol),
                            action: MenuAction::MusicVolume,
                        },
                        MenuItem {
                            text: vol_text("Sfx", aud.plr.sfx_vol),
                            action: MenuAction::SfxVolume,
                        },
                        MenuItem {
                            text: "Desktop fullscreen".into(),
                            action: MenuAction::DesktopFullscreen,
                        },
                        MenuItem {
                            text: "Back".into(),
                            action: MenuAction::Back,
                        },
                    ];
                    game.ui.menu.stack.push(items);
                    game.ui.menu.cursor = 0;
                }
            }
            MenuAction::Input => {
                if enter {
                    game.ui.menu.stack.push(build_keyconfig_menu(input));
                    game.ui.menu.cursor = 0;
                }
            }
            MenuAction::Rebind(act) => {
                if enter {
                    game.ui.menu.action_to_rebind = Some(*act);
                }
            }
            MenuAction::MusicVolume => {
                if left {
                    cmd.push(Cmd::MusVolDec);
                } else if right {
                    cmd.push(Cmd::MusVolInc)
                }
                current_menu_item.text = vol_text("Music", aud.mus_vol);
            }
            MenuAction::SfxVolume => {
                if left {
                    cmd.push(Cmd::SfxVolDec);
                } else if right {
                    cmd.push(Cmd::SfxVolInc)
                }
                current_menu_item.text = vol_text("Sfx", aud.plr.sfx_vol);
            }
            MenuAction::DesktopFullscreen => {
                if enter {
                    cmd.push(Cmd::DesktopFullscreen);
                }
            }
        }
    }
    if input.pressed_raw(Key::Escape) && !game.ui.menu.first_frame {
        game.ui.menu.cursor = 0;
        game.ui.menu.stack.pop();
        dbg!(&game.ui.menu.stack.len());
        if game.ui.menu.stack.is_empty() {
            game.ui.menu.open = false;
        }
    }
    #[expect(clippy::collapsible_if)]
    if input.pressed_raw(Key::Up) {
        if game.ui.menu.cursor > 0 {
            game.ui.menu.cursor -= 1;
        }
    }
    if let Some(list) = game.ui.menu.stack.last() {
        #[expect(clippy::collapsible_if)]
        if input.pressed_raw(Key::Down) {
            if game.ui.menu.cursor + 1 < list.len() {
                game.ui.menu.cursor += 1;
            }
        }
    }
    game.ui.menu.first_frame = false;
}

fn vol_text(pre: &str, mus_vol: f32) -> String {
    format!("« {pre} volume: {:.0}% »", mus_vol * 100.)
}

fn build_keyconfig_menu(input: &Input) -> Vec<MenuItem> {
    let mut items = Vec::new();
    for (action, key) in &input.key_bindings {
        items.push(MenuItem {
            text: format!("{}: {key:?}", action.name()),
            action: MenuAction::Rebind(*action),
        })
    }
    items.push(MenuItem {
        text: "Back".into(),
        action: MenuAction::Back,
    });
    items
}

pub fn open_menu(game: &mut GameState) {
    let list = vec![
        MenuItem {
            text: "New world (random)".into(),
            action: MenuAction::NewRandom,
        },
        MenuItem {
            text: "Load world".into(),
            action: MenuAction::Load,
        },
        MenuItem {
            text: "Settings".into(),
            action: MenuAction::Settings,
        },
        MenuItem {
            text: "Quit".into(),
            action: MenuAction::Quit,
        },
    ];
    game.ui.menu.stack.push(list);
    game.ui.menu.open = true;
    game.ui.menu.first_frame = true;
}

pub struct Menu {
    pub first_frame: bool,
    pub stack: MenuStack,
    pub cursor: usize,
    pub open: bool,
    pub action_to_rebind: Option<InputAction>,
    pub sel_color: Color,
}

impl Default for Menu {
    fn default() -> Self {
        Self {
            first_frame: true,
            stack: Default::default(),
            cursor: Default::default(),
            open: Default::default(),
            action_to_rebind: Default::default(),
            sel_color: Color::YELLOW,
        }
    }
}
