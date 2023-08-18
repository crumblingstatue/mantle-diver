use {
    crate::{debug::DBG_OVR, game::GameState, player::MovingEnt},
    sfml::graphics::{
        CircleShape, Color, PrimitiveType, RectangleShape, RenderStates, RenderTarget,
        RenderTexture, Shape, Transformable, Vertex,
    },
};

pub(crate) fn draw_debug_overlay(rt: &mut RenderTexture, game: &GameState) {
    let mut rs = RectangleShape::new();
    let (cx, cy) = (game.camera_offset.x, game.camera_offset.y);
    let mut cs = CircleShape::new(0., 30);
    DBG_OVR.for_each(|msg| match msg {
        crate::debug::DbgOvr::WldRect { r, c } => {
            let (Some(x), Some(y)) = (r.topleft.x.checked_sub(cx), r.topleft.y.checked_sub(cy))
            else {
                return;
            };
            rs.set_position((x as f32, y as f32));
            rs.set_size((r.w as f32, r.h as f32));
            rs.set_outline_color(*c);
            rs.set_fill_color(Color::TRANSPARENT);
            rs.set_outline_thickness(1.0);
            rt.draw(&rs);
        }
        crate::debug::DbgOvr::WldCircle { pos, radius, c } => {
            let (Some(x), Some(y)) = (pos.x.checked_sub(cx), pos.y.checked_sub(cy)) else {
                return;
            };
            let rad = f32::from(*radius);
            cs.set_origin((rad, rad));
            cs.set_position((x as f32, y as f32));
            cs.set_radius(rad);
            cs.set_outline_color(*c);
            cs.set_fill_color(Color::TRANSPARENT);
            cs.set_outline_thickness(1.0);
            rt.draw(&cs);
        }
        crate::debug::DbgOvr::WldLine { p1, p2 } => {
            let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
                p1.x.checked_sub(cx),
                p1.y.checked_sub(cy),
                p2.x.checked_sub(cx),
                p2.y.checked_sub(cy),
            ) else {
                return;
            };
            rt.draw_primitives(
                &[
                    Vertex::new(
                        (x1 as f32, y1 as f32).into(),
                        Color::YELLOW,
                        (0., 0.).into(),
                    ),
                    Vertex::new(
                        (x2 as f32, y2 as f32).into(),
                        Color::YELLOW,
                        (0., 0.).into(),
                    ),
                ],
                PrimitiveType::LINES,
                &RenderStates::default(),
            )
        }
    });
}

pub fn draw_controlled_en_bb(game: &mut GameState, rt: &mut RenderTexture) {
    let Ok(mov) = game.ecw.query_one_mut::<&MovingEnt>(game.controlled_en) else {
        return;
    };
    let (x, y, w, h) = mov.mob.en.xywh();
    let mut rect_sh = RectangleShape::new();
    rect_sh.set_fill_color(Color::rgba(255, 255, 255, 96));
    let (co_x, co_y) = game.camera_offset.to_signed();
    rect_sh.set_position(((x - co_x) as f32, (y - co_y) as f32));
    rect_sh.set_size((w as f32, h as f32));
    rt.draw(&rect_sh);
    rect_sh.set_size((2., 2.));
    rect_sh.set_fill_color(Color::RED);
    rect_sh.set_position((
        (mov.mob.en.pos.x - co_x) as f32,
        (mov.mob.en.pos.y - co_y) as f32,
    ));
    rt.draw(&rect_sh);
}
