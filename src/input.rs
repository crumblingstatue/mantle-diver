use {
    crate::math::ScreenVecExt,
    fnv::{FnvHashMap, FnvHashSet},
    mdv_math::types::ScreenVec,
    sfml::window::{mouse, Event, Key},
};

pub struct Input {
    down: FnvHashSet<Key>,
    pressed: FnvHashSet<Key>,
    pub key_bindings: FnvHashMap<InputAction, Key>,
    pub lmb_down: bool,
    pub rmb_down: bool,
    pub mouse_down_loc: ScreenVec,
    pub mid_pressed: bool,
    pub just_pressed_raw: Option<Key>,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            down: Default::default(),
            pressed: Default::default(),
            key_bindings: default_key_bindings(),
            lmb_down: Default::default(),
            rmb_down: Default::default(),
            mouse_down_loc: Default::default(),
            mid_pressed: Default::default(),
            just_pressed_raw: None,
        }
    }
}

fn default_key_bindings() -> FnvHashMap<InputAction, Key> {
    let mut hm = FnvHashMap::default();
    hm.insert(InputAction::Down, Key::S);
    hm.insert(InputAction::Left, Key::A);
    hm.insert(InputAction::Up, Key::W);
    hm.insert(InputAction::Right, Key::D);
    hm.insert(InputAction::Jump, Key::Space);
    hm
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
    #[expect(dead_code, reason = "Will probably use later")]
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
}

impl InputAction {
    pub fn name(&self) -> &'static str {
        match self {
            InputAction::Left => "Left",
            InputAction::Right => "Right",
            InputAction::Jump => "Jump",
            InputAction::Down => "Down",
            InputAction::Up => "Up",
        }
    }
}
