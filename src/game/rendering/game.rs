use {
    super::debug::draw_controlled_en_bb,
    crate::{
        debug::DebugState,
        game::{for_each_tile_on_screen, Biome, GameState, TilestateKey},
        light::{self, LightEnumInfo, U16Vec},
        math::{IntRectExt, ScreenVecExt, WorldPos, TILE_SIZE},
        player::{FacingDir, MoveExtra, MovingEnt, PlayerColors},
        res::Res,
        sfml::{SpriteExt, WorldRectExt},
    },
    mdv_data::{item::ItemId, tile::LayerAccess},
    mdv_math::types::ScreenVec,
    sfml::graphics::{
        Color, PrimitiveType, Rect, RectangleShape, RenderStates, RenderTarget, RenderTexture,
        Shape, Sprite, Transform, Transformable, Vertex,
    },
};

pub(crate) fn draw_world(
    game: &mut GameState,
    verts: &mut Vec<Vertex>,
    rt: &mut RenderTexture,
    res: &Res,
) {
    verts.clear();
    match game.current_biome {
        Biome::Surface => {
            let mut s = Sprite::with_texture(&res.forest_bg);
            s.fit_to_size(rt.size().as_other());
            let c = game.ambient_light;
            s.set_color(Color::rgb(c, c, c));
            rt.draw(&s);
        }
        Biome::Underground => rt.clear(Color::rgb(0, 10, 30)),
    }
    for_each_tile_on_screen(
        game.camera_offset,
        ScreenVec::from_sf_resolution(rt.size()),
        |tp, sp| {
            let tile = *game.world.tile_at_mut(tp);
            let color = Color::WHITE;
            let x = f32::from(sp.x);
            let y = f32::from(sp.y);
            if !tile.bg.empty() {
                let def = &game.tile_db[tile.bg];
                let tx = def.tex_rect.x as f32;
                let ty = def.tex_rect.y as f32;
                let tw = def.tex_rect.w as f32;
                let th = def.tex_rect.h as f32;
                verts.push(Vertex::new((x, y).into(), color, (tx, ty).into()));
                verts.push(Vertex::new((x + tw, y).into(), color, (tx + tw, ty).into()));
                verts.push(Vertex::new(
                    (x + tw, y + th).into(),
                    color,
                    (tx + tw, ty + th).into(),
                ));
                verts.push(Vertex::new((x, y + th).into(), color, (tx, ty + th).into()));
            }
            if !tile.mid.empty() {
                let def = &game.tile_db[tile.mid];
                let x = x + f32::from(def.draw_offs.x);
                let y = y + f32::from(def.draw_offs.y);
                let mut rect = def.tex_rect.to_sf();
                // Tile blending prototype code
                let rect = if !def.neigh_aware {
                    rect
                } else {
                    rect.width = i32::from(TILE_SIZE);
                    rect.height = i32::from(TILE_SIZE);
                    let left = def
                        .blend_list
                        .contains(&game.world.tile_at_mut(tp.x_off(-1)).mid.0);
                    let right = def
                        .blend_list
                        .contains(&game.world.tile_at_mut(tp.x_off(1)).mid.0);
                    let above = def
                        .blend_list
                        .contains(&game.world.tile_at_mut(tp.y_off(-1)).mid.0);
                    let below = def
                        .blend_list
                        .contains(&game.world.tile_at_mut(tp.y_off(1)).mid.0);
                    adjust_blend_rect(&mut rect, left, right, above, below);
                    rect
                };
                let mut rot = 0.;
                let mut scale = 1.;
                if let Some(state) = game.transient_tile_states.get(&TilestateKey {
                    pos: tp,
                    layer: LayerAccess::Mid,
                }) {
                    rot = state.rot;
                    scale = state.scale;
                }
                let tx = rect.left as f32;
                let ty = rect.top as f32;
                let tw = rect.width as f32;
                let th = rect.height as f32;
                let mut tf = Transform::default();
                tf.scale_with_center(scale, scale, x, y);
                tf.rotate_with_center(rot, x, y);
                verts.push(Vertex::new(
                    tf.transform_point((x, y).into()),
                    color,
                    (tx, ty).into(),
                ));
                verts.push(Vertex::new(
                    tf.transform_point((x + tw, y).into()),
                    color,
                    (tx + tw, ty).into(),
                ));
                verts.push(Vertex::new(
                    tf.transform_point((x + tw, y + th).into()),
                    color,
                    (tx + tw, ty + th).into(),
                ));
                verts.push(Vertex::new(
                    tf.transform_point((x, y + th).into()),
                    color,
                    (tx, ty + th).into(),
                ));
            }
        },
    );
    let mut rs = RenderStates::DEFAULT;
    rs.set_texture(Some(&res.atlas.tex));
    rt.draw_primitives(verts, PrimitiveType::QUADS, &rs);
    // Draw tile highlight rect
    if let Some(hi_rect) = &game.highlight_tp {
        let sf_rect = hi_rect.tile_world_rect().to_sf_draw(game.camera_offset);
        let mut rs = RectangleShape::from_rect(sf_rect);
        rs.set_outline_color(Color::YELLOW);
        rs.set_outline_thickness(-1.0);
        rs.set_fill_color(Color::rgba(227, 200, 0, 128));
        rt.draw(&rs);
    }
}
pub fn draw_entities(game: &mut GameState, rt: &mut RenderTexture, res: &Res, debug: &DebugState) {
    let mut s = Sprite::with_texture(&res.atlas.tex);
    s.set_origin((16., 16.));
    for (_en, (id, mov)) in game.ecw.query_mut::<(&ItemId, &MovingEnt)>() {
        let pos = mov.mob.en.pos;
        let item_def = &game.itemdb.get(*id).unwrap();
        let (co_x, co_y) = game.camera_offset.to_signed();
        s.set_position(((pos.x - co_x) as f32, (pos.y - co_y) as f32));
        s.set_texture_rect(item_def.tex_rect.to_sf());
        rt.draw(&s);
    }
    draw_player(game, rt, debug, res);
}

fn draw_player(game: &mut GameState, rt: &mut RenderTexture, debug: &DebugState, res: &Res) {
    draw_player_sprites(game, rt, res);
    if debug.dbg_overlay {
        draw_controlled_en_bb(game, rt);
    }
}

fn draw_player_sprites(game: &mut GameState, rt: &mut RenderTexture, res: &Res) {
    let Ok((mov, mov_extra, colors)) = game
        .ecw
        .query_one_mut::<(&MovingEnt, &MoveExtra, &PlayerColors)>(game.player_en)
    else {
        log::error!("Player query failed");
        return;
    };
    let mut s = Sprite::with_texture(&res.atlas.tex);
    let (x, y, _w, _h) = mov.mob.en.xywh();
    let (co_x, co_y) = game.camera_offset.to_signed();
    let (base_x, base_y) = ((x - co_x) as f32, (y - co_y) as f32);
    let held_item_id = game.inventory.slots[game.ui.selected_inv_slot].id;
    let held_item_graphic = game.itemdb.get(held_item_id).map(|def| &def.graphic_name);
    let mut drawable_tool = false;
    let (
        mut head_x,
        mut eye_x,
        mut hair_x,
        mut torso_x,
        mut legs_x,
        mut tool_x,
        mut tool_y,
        mut pants_x,
        mut shirt_x,
        mut shoes_x,
    ) = (0.0, 4.0, -4.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    match mov_extra.facing_dir {
        FacingDir::Left => {
            if let Some(offs) = game.char_db.graphic_offsets.get("char/head1") {
                head_x = f32::from(offs.left.x);
            }
            if let Some(offs) = game.char_db.graphic_offsets.get("char/eye1") {
                eye_x = f32::from(offs.left.x);
            }
            if let Some(offs) = game.char_db.graphic_offsets.get("char/hair1") {
                hair_x = f32::from(offs.left.x);
            }
            if let Some(offs) = game.char_db.graphic_offsets.get("char/torso1") {
                torso_x = f32::from(offs.left.x);
            }
            if let Some(offs) = game.char_db.graphic_offsets.get("char/legs1") {
                legs_x = f32::from(offs.left.x);
            }
            if let Some(offs) = game.char_db.graphic_offsets.get("char/pants1") {
                pants_x = f32::from(offs.left.x);
            }
            if let Some(offs) = game.char_db.graphic_offsets.get("char/shirt1") {
                shirt_x = f32::from(offs.left.x);
            }
            if let Some(offs) = game.char_db.graphic_offsets.get("char/shoes1") {
                shoes_x = f32::from(offs.left.x);
            }
            if let Some(graphic) = held_item_graphic
                && let Some(offs) = game.char_db.graphic_offsets.get(graphic)
            {
                tool_x = f32::from(offs.left.x);
                tool_y = f32::from(offs.left.y);
                drawable_tool = true;
            }
        }
        FacingDir::Right => {
            s.set_scale((-1.0, 1.0));
            if let Some(offs) = game.char_db.graphic_offsets.get("char/head1") {
                head_x = f32::from(offs.right.x);
            }
            if let Some(offs) = game.char_db.graphic_offsets.get("char/eye1") {
                eye_x = f32::from(offs.right.x);
            }
            if let Some(offs) = game.char_db.graphic_offsets.get("char/hair1") {
                hair_x = f32::from(offs.right.x);
            }
            if let Some(offs) = game.char_db.graphic_offsets.get("char/torso1") {
                torso_x = f32::from(offs.right.x);
            }
            if let Some(offs) = game.char_db.graphic_offsets.get("char/legs1") {
                legs_x = f32::from(offs.right.x);
            }
            if let Some(offs) = game.char_db.graphic_offsets.get("char/pants1") {
                pants_x = f32::from(offs.right.x);
            }
            if let Some(offs) = game.char_db.graphic_offsets.get("char/shirt1") {
                shirt_x = f32::from(offs.right.x);
            }
            if let Some(offs) = game.char_db.graphic_offsets.get("char/shoes1") {
                shoes_x = f32::from(offs.right.x);
            }
            if let Some(graphic) = held_item_graphic
                && let Some(offs) = game.char_db.graphic_offsets.get(graphic)
            {
                tool_x = f32::from(offs.right.x);
                tool_y = f32::from(offs.right.y);
                drawable_tool = true;
            }
        }
    }
    s.set_color(colors.skin);
    // Head
    s.set_texture_rect(res.atlas.rects["char/head1"].to_sf());
    s.set_position((base_x + head_x, base_y));
    rt.draw(&s);
    // Eye
    s.set_color(colors.eye);
    s.set_texture_rect(res.atlas.rects["char/eye1"].to_sf());
    s.set_position((base_x + eye_x, base_y + 9.));
    rt.draw(&s);
    // Hair
    s.set_color(colors.hair);
    s.set_texture_rect(res.atlas.rects["char/hair1"].to_sf());
    s.set_position((base_x + hair_x, base_y - 4.));
    rt.draw(&s);
    s.set_color(colors.skin);
    // Torso
    s.set_texture_rect(res.atlas.rects["char/torso1"].to_sf());
    s.set_position((base_x + torso_x, base_y + 32.0));
    rt.draw(&s);
    // Legs
    s.set_texture_rect(res.atlas.rects["char/legs1"].to_sf());
    s.set_position((base_x + legs_x, base_y + 64.0));
    rt.draw(&s);
    // Shirt
    s.set_texture_rect(res.atlas.rects["char/shirt1"].to_sf());
    s.set_position((base_x + shirt_x, base_y + 32.0));
    s.set_color(colors.shirt);
    rt.draw(&s);
    // Pants
    s.set_texture_rect(res.atlas.rects["char/pants1"].to_sf());
    s.set_position((base_x + pants_x, base_y + 64.0));
    s.set_color(colors.pants);
    rt.draw(&s);
    // Shoes
    s.set_texture_rect(res.atlas.rects["char/shoes1"].to_sf());
    s.set_position((base_x + shoes_x, base_y + 87.0));
    s.set_color(colors.shoes);
    rt.draw(&s);
    // Tool
    if let Some(graphic) = held_item_graphic
        && drawable_tool
    {
        s.set_color(Color::WHITE);
        s.set_texture_rect(res.atlas.rects[graphic].to_sf());
        s.set_position((base_x + tool_x, base_y + tool_y));
    }
    rt.draw(&s);
}

fn adjust_blend_rect(rect: &mut Rect<i32>, left: bool, right: bool, above: bool, below: bool) {
    /*
         0 1 2 3 x
       0 X - - -
       1 | [ - ]
       2 | |   |
       3 | [ _ ]
       y
    */
    let (x, y) = match (left, right, above, below) {
        // l   r     a     b
        (true, true, true, true) => (2, 2),
        (true, true, true, false) => (2, 3),
        (true, true, false, true) => (2, 1),
        (true, true, false, false) => (2, 0),
        (true, false, true, true) => (3, 2),
        (true, false, true, false) => (3, 3),
        (true, false, false, true) => (3, 1),
        (true, false, false, false) => (3, 0),
        (false, true, true, true) => (1, 2),
        (false, true, true, false) => (1, 3),
        (false, true, false, true) => (1, 1),
        (false, true, false, false) => (1, 0),
        (false, false, true, true) => (0, 2),
        (false, false, true, false) => (0, 3),
        (false, false, false, true) => (0, 1),
        (false, false, false, false) => (0, 0),
    };
    rect.left += x * i32::from(TILE_SIZE);
    rect.top += y * i32::from(TILE_SIZE);
}

pub(crate) fn light_blend_pass(
    camera_offset: WorldPos,
    verts: &mut Vec<Vertex>,
    lt_tex: &mut RenderTexture,
    lightmap: &[u8],
    tiles_on_screen: U16Vec,
    light_enum_info: LightEnumInfo,
) {
    verts.clear();
    lt_tex.clear(Color::TRANSPARENT);
    let xoff = (camera_offset.x % u32::from(TILE_SIZE)) as f32;
    let yoff = (camera_offset.y % u32::from(TILE_SIZE)) as f32;
    for y in 0..tiles_on_screen.y {
        for x in 0..tiles_on_screen.x {
            let lmx = usize::from(x) + usize::from(light::MAX_TILE_REACH);
            let lmy = usize::from(y) + usize::from(light::MAX_TILE_REACH);
            let i = lmy * usize::from(light_enum_info.width) + lmx;
            let level = lightmap[i];
            let x = f32::from(x) * f32::from(TILE_SIZE);
            let y = f32::from(y) * f32::from(TILE_SIZE);
            let c = Color::rgba(255, 255, 255, level);
            let x = x - xoff;
            let y = y - yoff;
            let ts = f32::from(TILE_SIZE);
            verts.push(Vertex::new((x, y).into(), c, (0., 0.).into()));
            verts.push(Vertex::new((x + ts, y).into(), c, (0., 0.).into()));
            verts.push(Vertex::new((x + ts, y + ts).into(), c, (0., 0.).into()));
            verts.push(Vertex::new((x, y + ts).into(), c, (0., 0.).into()));
        }
    }
    lt_tex.draw_primitives(verts, PrimitiveType::QUADS, &RenderStates::DEFAULT);
}
