use {
    crate::math::FPS_TARGET,
    mdv_math::types::{ScreenSc, ScreenVec},
    sfml::{
        graphics::RenderWindow,
        window::{ContextSettings, Style},
        SfBox,
    },
    sfml_xt::graphics::RenderWindowExt,
};

#[derive(Clone, Copy)]
pub struct ScreenRes {
    pub w: u16,
    pub h: u16,
}
impl ScreenRes {
    #[expect(clippy::cast_possible_wrap, reason = "Never exceeds i16::MAX")]
    pub(crate) fn to_vec(self) -> mdv_math::types::ScreenVec {
        ScreenVec {
            x: self.w as ScreenSc,
            y: self.h as ScreenSc,
        }
    }
}

const DEFAULT_RES: ScreenRes = ScreenRes { w: 960, h: 540 };

pub fn make_window() -> SfBox<RenderWindow> {
    let mut rw = RenderWindow::new(
        DEFAULT_RES.to_sf(),
        "Mantle Diver",
        Style::DEFAULT,
        &ContextSettings::default(),
    )
    .unwrap();
    rw.set_framerate_limit(FPS_TARGET.into());
    rw.center();
    rw
}
