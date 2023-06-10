use {
    crate::math::FPS_TARGET,
    serde::{Deserialize, Serialize},
    sfml::{
        graphics::RenderWindow,
        system::Vector2f,
        window::{ContextSettings, Style, VideoMode},
    },
    sfml_xt::graphics::RenderWindowExt,
    std::ops::Div,
};

pub struct ScreenRes {
    pub w: u16,
    pub h: u16,
}

impl ScreenRes {
    fn to_sf(&self) -> VideoMode {
        VideoMode {
            width: self.w.into(),
            height: self.h.into(),
            bits_per_pixel: 32,
        }
    }
}

#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ScreenVec {
    pub x: ScreenSc,
    pub y: ScreenSc,
}

/// Screen position/offset scalar
/// We assume this game won't be played above 32767*32767 resolution
pub type ScreenSc = i16;

impl ScreenVec {
    pub fn to_sf_vec(self) -> Vector2f {
        Vector2f::new(self.x.into(), self.y.into())
    }

    pub(crate) fn from_sf2u(size: sfml::system::Vector2<u32>) -> Self {
        Self {
            x: size.x as ScreenSc,
            y: size.y as ScreenSc,
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

const DEFAULT_RES: ScreenRes = ScreenRes { w: 960, h: 540 };

pub fn make_window() -> RenderWindow {
    let mut rw = RenderWindow::new(
        DEFAULT_RES.to_sf(),
        "Mantle Diver",
        Style::DEFAULT,
        &ContextSettings::default(),
    );
    rw.set_framerate_limit(FPS_TARGET.into());
    rw.center();
    rw
}
