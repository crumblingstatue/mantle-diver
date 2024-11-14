use {
    crate::{
        graphics::ScreenRes,
        math::{WorldPos, WorldRect},
    },
    egui_sfml::{egui, SfEgui},
    extension_traits::extension,
    mdv_math::types::{ScreenRect, ScreenVec},
    sfml::{
        graphics::{FloatRect, RectangleShape, RenderTarget, RenderWindow, Sprite, Transformable},
        system::{Vector2f, Vector2u},
        window::{ContextSettings, Event, Style, VideoMode},
    },
    sfml_xt::graphics::RenderWindowExt,
};

pub trait SpriteExt {
    fn fit_to_size(&mut self, size: Vector2f);
}

impl SpriteExt for Sprite<'_> {
    fn fit_to_size(&mut self, size: Vector2f) {
        let Some(tex) = self.texture() else { return };
        let self_size: Vector2f = tex.size().as_other();
        self.set_scale((size.x / self_size.x, size.y / self_size.y));
    }
}

impl ScreenRes {
    pub fn to_sf(self) -> VideoMode {
        VideoMode {
            width: self.w.into(),
            height: self.h.into(),
            bits_per_pixel: 32,
        }
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "This should be used for window and texture sizes, which shouldn't exceed u16::MAX"
    )]
    pub fn from_sf_vec(vec: Vector2u) -> Self {
        Self {
            w: vec.x as u16,
            h: vec.y as u16,
        }
    }
}

pub trait SfVec2fExt {
    fn scv_off(&self, off: ScreenVec) -> Self;
}

impl SfVec2fExt for Vector2f {
    fn scv_off(&self, off: ScreenVec) -> Self {
        Self {
            x: self.x + f32::from(off.x),
            y: self.y + f32::from(off.y),
        }
    }
}

#[extension(pub trait RenderTargetExt)]
impl<T: RenderTarget> T {
    fn res(&self) -> ScreenRes {
        let size = self.size();
        #[expect(
            clippy::cast_possible_truncation,
            reason = "Size is smaller than i16::MAX"
        )]
        ScreenRes {
            w: size.x as u16,
            h: size.y as u16,
        }
    }
}

#[extension(pub trait ScreenRectSfExt)]
impl ScreenRect {
    fn into_sf(&self) -> FloatRect {
        FloatRect {
            left: f32::from(self.x),
            top: f32::from(self.y),
            width: f32::from(self.w),
            height: f32::from(self.h),
        }
    }
    fn sf_position(&self) -> Vector2f {
        Vector2f::new(f32::from(self.x), f32::from(self.y))
    }
}

#[extension(pub trait ScreenVecSfExt)]
impl ScreenVec {
    fn to_sf_vec2f(&self) -> Vector2f {
        Vector2f {
            x: f32::from(self.x),
            y: f32::from(self.y),
        }
    }
}

#[extension(pub trait RectangleShapeExt)]
impl RectangleShape<'_> {
    fn set_screen_rect(&mut self, rect: ScreenRect) {
        self.set_position((f32::from(rect.x), f32::from(rect.y)));
        self.set_size((f32::from(rect.w), f32::from(rect.h)));
    }
}

#[extension(pub trait WorldRectExt)]
impl WorldRect {
    fn to_sf_draw(&self, camera_off: WorldPos) -> FloatRect {
        let pos = self.topleft - camera_off;
        FloatRect {
            left: pos.x as f32,
            top: pos.y as f32,
            width: self.w as f32,
            height: self.h as f32,
        }
    }
}

pub fn show_fatal_error_window(title: &str, body: impl AsRef<str>) {
    let mut rw = RenderWindow::new(
        (800, 600),
        title,
        Style::default(),
        &ContextSettings::default(),
    )
    .unwrap();
    rw.set_framerate_limit(60);
    rw.center();
    let mut sf_egui = SfEgui::new(&rw);
    while rw.is_open() {
        while let Some(ev) = rw.poll_event() {
            sf_egui.add_event(&ev);
            if ev == Event::Closed {
                rw.close();
            }
        }
        let di = sf_egui
            .run(&mut rw, |_rw, ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(title);
                    });
                    ui.label(body.as_ref());
                });
            })
            .unwrap();
        sf_egui.draw(di, &mut rw, None);
        rw.display();
    }
}
