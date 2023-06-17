use {
    crate::{
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
}

pub fn pause_menu_system(
    game: &mut GameState,
    input: &mut Input,
    cmd: &mut CmdVec,
    worlds_dir: &Path,
) {
    if let Some(act) = game.menu.action_to_rebind {
        game.menu.sel_color = Color::RED;
        if let Some(key) = input.just_pressed_raw {
            input.key_bindings.insert(act, key);
            game.menu.action_to_rebind = None;
            if let Some(items) = game.menu.stack.last_mut() {
                *items = build_keyconfig_menu(input);
            }
        }
        return;
    }
    game.menu.sel_color = Color::YELLOW;
    if input.pressed_raw(Key::Enter) {
        if let Some(list) = game.menu.stack.last() {
            match &list[game.menu.cursor].action {
                MenuAction::NewRandom => {
                    let n: u32 = thread_rng().gen();
                    cmd.push(Cmd::LoadWorld(n.to_string()));
                }
                MenuAction::Load => {
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
                    game.menu.stack.push(list);
                    game.menu.cursor = 0;
                }
                MenuAction::Quit => cmd.push(Cmd::QuitApp),
                MenuAction::LoadWorld(name) => cmd.push(Cmd::LoadWorld(name.clone())),
                MenuAction::Back => {
                    game.menu.cursor = 0;
                    game.menu.stack.pop();
                    if game.menu.stack.is_empty() {
                        game.menu.open = false;
                    }
                }
                MenuAction::Settings => {
                    let items = vec![
                        MenuItem {
                            text: "Input".into(),
                            action: MenuAction::Input,
                        },
                        MenuItem {
                            text: "Back".into(),
                            action: MenuAction::Back,
                        },
                    ];
                    game.menu.stack.push(items);
                    game.menu.cursor = 0;
                }
                MenuAction::Input => {
                    game.menu.stack.push(build_keyconfig_menu(input));
                    game.menu.cursor = 0;
                }
                MenuAction::Rebind(act) => {
                    game.menu.action_to_rebind = Some(*act);
                }
            }
        }
    }
    if input.pressed_raw(Key::Escape) && !game.menu.first_frame {
        game.menu.cursor = 0;
        game.menu.stack.pop();
        dbg!(&game.menu.stack.len());
        if game.menu.stack.is_empty() {
            game.menu.open = false;
        }
    }
    #[expect(clippy::collapsible_if)]
    if input.pressed_raw(Key::Up) {
        if game.menu.cursor > 0 {
            game.menu.cursor -= 1;
        }
    }
    if let Some(list) = game.menu.stack.last() {
        #[expect(clippy::collapsible_if)]
        if input.pressed_raw(Key::Down) {
            if game.menu.cursor + 1 < list.len() {
                game.menu.cursor += 1;
            }
        }
    }
    game.menu.first_frame = false;
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
    game.menu.stack.push(list);
    game.menu.open = true;
    game.menu.first_frame = true;
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
