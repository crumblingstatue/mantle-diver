use {
    crate::graphics::ScreenVec,
    fnv::FnvHashSet,
    sfml::window::{mouse, Event, Key},
};

#[derive(Default, Debug)]
pub struct Input {
    down: FnvHashSet<Key>,
    pressed: FnvHashSet<Key>,
    pub lmb_down: bool,
    pub rmb_down: bool,
    pub mouse_down_loc: ScreenVec,
    pub mid_pressed: bool,
}

impl Input {
    pub fn update_from_event(&mut self, ev: &Event, egui_kbd: bool, egui_ptr: bool) {
        match ev {
            &Event::KeyPressed { code, .. } => {
                self.pressed.insert(code);
                self.down.insert(code);
            }
            Event::KeyReleased { code, .. } => {
                self.down.remove(code);
            }
            &Event::MouseButtonPressed { button, x, y } => {
                self.mouse_down_loc = ScreenVec {
                    x: x as i16,
                    y: y as i16,
                };
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
            &Event::MouseMoved { x, y } => {
                self.mouse_down_loc.x = x as i16;
                self.mouse_down_loc.y = y as i16;
            }
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
    }
    pub fn down(&self, key: Key) -> bool {
        self.down.contains(&key)
    }
    pub fn pressed(&self, key: Key) -> bool {
        self.pressed.contains(&key)
    }
}
