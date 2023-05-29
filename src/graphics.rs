use {
    crate::math::FPS_TARGET,
    serde::{Deserialize, Serialize},
    sfml::{
        graphics::RenderWindow,
        system::Vector2f,
        window::{ContextSettings, Style, VideoMode},
    },
    sfml_xt::graphics::RenderWindowExt,
};

pub struct ScreenRes {
    pub w: u16,
    pub h: u16,
}

impl ScreenRes {
    fn to_sf(&self) -> VideoMode {
        VideoMode {
            width: self.w as _,
            height: self.h as _,
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
