use {
    crate::math::ScreenVecExt,
    fnv::{FnvHashMap, FnvHashSet},
    mdv_math::types::ScreenVec,
    sfml::window::{mouse, Event, Key},
    smart_default::SmartDefault,
};

#[derive(SmartDefault)]
pub struct Input {
    down: FnvHashSet<Key>,
    pressed: FnvHashSet<Key>,
    #[default(default_key_bindings())]
    pub key_bindings: FnvHashMap<InputAction, Key>,
    pub lmb_down: bool,
    pub lmb_pressed: bool,
    pub rmb_down: bool,
    pub mouse_down_loc: ScreenVec,
    pub mid_pressed: bool,
    pub just_pressed_raw: Option<Key>,
}

fn default_key_bindings() -> FnvHashMap<InputAction, Key> {
    FnvHashMap::from_iter([
        (InputAction::Down, Key::S),
        (InputAction::Left, Key::A),
        (InputAction::Up, Key::W),
        (InputAction::Right, Key::D),
        (InputAction::Jump, Key::Space),
        (InputAction::ThrowItem, Key::Q),
        (InputAction::Interact, Key::E),
        (InputAction::SelectTorch, Key::LShift),
        (InputAction::Inventory, Key::I),
        (InputAction::Crafting, Key::C),
        (InputAction::SmartCursorToggle, Key::LControl),
    ])
}

impl Input {
    pub fn update_from_event(&mut self, ev: &Event, egui_kbd: bool, egui_ptr: bool) {
        match ev {
            &Event::KeyPressed { code, .. } => {
                self.pressed.insert(code);
                self.down.insert(code);
                self.just_pressed_raw = Some(code);
            }
            Event::KeyReleased { code, .. } => {
                self.down.remove(code);
            }
            &Event::MouseButtonPressed { button, x, y } => {
                self.mouse_down_loc = ScreenVec::from_reso_i32(x, y);
                if button == mouse::Button::Left {
                    self.lmb_down = true;
                    self.lmb_pressed = true;
                }
                if button == mouse::Button::Right {
                    self.rmb_down = true;
                }
                if button == mouse::Button::Middle {
                    self.mid_pressed = true;
                }
            }
            &Event::MouseButtonReleased { button, .. } => {
                if button == mouse::Button::Left {
                    self.lmb_down = false;
                }
                if button == mouse::Button::Right {
                    self.rmb_down = false;
                }
            }
            &Event::MouseMoved { x, y } => self.mouse_down_loc = ScreenVec::from_reso_i32(x, y),
            _ => {}
        }
        if egui_kbd {
            self.pressed.clear();
            self.down.clear();
        }
        if egui_ptr {
            self.lmb_down = false;
            self.rmb_down = false;
            self.mid_pressed = false;
        }
    }
    /// Pressed event should be cleared every frame
    pub fn clear_pressed(&mut self) {
        self.mid_pressed = false;
        self.pressed.clear();
        self.just_pressed_raw = None;
        self.lmb_pressed = false;
    }
    pub fn down_raw(&self, key: Key) -> bool {
        self.down.contains(&key)
    }
    pub fn pressed_raw(&self, key: Key) -> bool {
        self.pressed.contains(&key)
    }
    pub fn down(&self, action: InputAction) -> bool {
        match self.key_bindings.get(&action) {
            Some(key) => self.down_raw(*key),
            None => {
                log::warn!("Missing key bind for {}", action.name());
                false
            }
        }
    }
    pub fn pressed(&self, action: InputAction) -> bool {
        self.pressed_raw(self.key_bindings[&action])
    }
    /// Clear all keyboard state
    pub(crate) fn clear_all_kbd(&mut self) {
        self.down.clear();
        self.pressed.clear();
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum InputAction {
    Left,
    Right,
    Jump,
    Down,
    Up,
    ThrowItem,
    Interact,
    SelectTorch,
    Inventory,
    Crafting,
    SmartCursorToggle,
}

impl InputAction {
    pub fn name(&self) -> &'static str {
        match self {
            InputAction::Left => "Left",
            InputAction::Right => "Right",
            InputAction::Jump => "Jump",
            InputAction::Down => "Down",
            InputAction::Up => "Up",
            InputAction::ThrowItem => "Throw item",
            InputAction::Interact => "Interact",
            InputAction::SelectTorch => "Select torch",
            InputAction::Inventory => "Inventory",
            InputAction::Crafting => "Crafting",
            InputAction::SmartCursorToggle => "Smart cursor toggle",
        }
    }
}
