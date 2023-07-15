use {
    serde::{Deserialize, Serialize},
    std::ops::Div,
};

/// Screen position/offset scalar
/// We assume this game won't be played above 32767*32767 resolution
pub type ScreenSc = i16;

/// A 2d screen coordinate vector
#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ScreenVec {
    pub x: ScreenSc,
    pub y: ScreenSc,
}
impl ScreenVec {
    /// Divided by the scale amount. Used to adjust window position to in-game position.
    pub fn scaled<T: Into<ScreenSc>>(self, scale: T) -> Self {
        let scale = scale.into();
        Self {
            x: self.x / scale,
            y: self.y / scale,
        }
    }
}

impl Div<ScreenSc> for ScreenVec {
    type Output = Self;

    fn div(self, rhs: ScreenSc) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
pub struct Rect<T> {
    pub x: T,
    pub y: T,
    pub w: T,
    pub h: T,
}

impl ScreenRect {
    pub fn contains_screen_pos(&self, pos: ScreenVec) -> bool {
        pos.x > self.x && pos.y > self.y && pos.x < self.x + self.w && pos.y < self.y + self.h
    }

    pub fn offset(self, x: ScreenSc, y: ScreenSc) -> Self {
        Self {
            x: self.x.saturating_add(x),
            y: self.y.saturating_add(y),
            w: self.w,
            h: self.h,
        }
    }
}

pub type IntRect = Rect<i32>;
pub type ScreenRect = Rect<ScreenSc>;
