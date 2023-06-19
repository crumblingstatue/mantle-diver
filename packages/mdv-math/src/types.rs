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
pub struct IntRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}