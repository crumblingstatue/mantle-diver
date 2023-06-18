use {
    crate::math::FPS_TARGET,
    sfml::{
        graphics::RenderWindow,
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
            width: self.w.into(),
            height: self.h.into(),
            bits_per_pixel: 32,
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
