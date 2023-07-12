use {
    super::{for_each_tile_on_screen, ui::Inventory, Biome, GameState, TilestateKey},
    crate::{
        config::Config,
        debug::{DebugState, DBG_OVR},
        light::{self, LightEnumInfo, U16Vec},
        math::{IntRectExt, ScreenVecExt, WorldPos, FPS_TARGET, TILE_SIZE},
        player::{FacingDir, Health, MoveExtra, MovingEnt, PlayerColors},
        res::Res,
        sfml::{RenderTargetExt, ScreenRectSfExt, SfVec2fExt, SpriteExt},
        stringfmt::LengthDisp,
        time::ticks_hm,
    },
    mdv_data::{item::ItemId, tile::LayerAccess},
    mdv_math::types::ScreenVec,
    sfml::{
        graphics::{
            CircleShape, Color, PrimitiveType, Rect, RectangleShape, RenderStates, RenderTarget,
            RenderTexture, Shape, Sprite, Text, Transform, Transformable, Vertex,
        },
        system::Vector2f,
    },
};

pub struct RenderState {
    /// Light map overlay, blended together with the non-lighted version of the scene
    pub light_blend_rt: RenderTexture,
    pub vert_array: Vec<Vertex>,
    /// RenderTexture for rendering the game at its native resolution
    pub rt: RenderTexture,
}

pub(crate) fn draw_world(
    game: &mut GameState,
    verts: &mut Vec<Vertex>,
    rt: &mut RenderTexture,
    res: &mut Res,
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
            if let Some(graphic) = held_item_graphic && let Some(offs) = game.char_db.graphic_offsets.get(graphic) {
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
            if let Some(graphic) = held_item_graphic && let Some(offs) = game.char_db.graphic_offsets.get(graphic) {
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
    if let Some(graphic) = held_item_graphic && drawable_tool {
        s.set_color(Color::WHITE);
        s.set_texture_rect(res.atlas.rects[graphic].to_sf());
        s.set_position((base_x + tool_x, base_y + tool_y));
    }
    rt.draw(&s);
}

fn draw_controlled_en_bb(game: &mut GameState, rt: &mut RenderTexture) {
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

pub fn draw_ui(
    game: &mut GameState,
    rt: &mut RenderTexture,
    res: &Res,
    ui_dims: Vector2f,
    cfg: &Config,
    debug: &DebugState,
) {
    if game.ui.inv.open {
        draw_inventory(game, cfg, rt, res);
    }
    let mut s = Sprite::with_texture(&res.atlas.tex);
    let mut rs = RectangleShape::from_rect(Rect::new(0., 0., 36., 36.));
    let mut text = Text::new("", &res.sans_font, 14);
    if let Some(def) = game
        .itemdb
        .get(game.inventory.slots[game.ui.selected_inv_slot].id)
    {
        text.set_outline_thickness(1.0);
        text.set_outline_color(Color::BLACK);
        text.set_position((4., ui_dims.y - 70.0));
        text.set_string(&def.name);
        rt.draw(&text);
        text.set_outline_thickness(0.0);
    }
    let inv_frame_color = cfg.ui.inv_frame_color.to_sf();
    let inv_frame_highlight = cfg.ui.inv_frame_highlight.to_sf();
    let inv_bg_color = cfg.ui.inv_bg_color.to_sf();
    for (i, slot) in game.inventory.slots.iter().enumerate() {
        let pos = ((i * 44) as f32 + 8.0, (ui_dims.y - 48.));
        rs.set_position((pos.0 + 2., pos.1 + 2.));
        rs.set_fill_color(inv_bg_color);
        if i == game.ui.selected_inv_slot {
            s.set_color(inv_frame_highlight);
        } else {
            rs.set_outline_thickness(0.0);
            s.set_color(inv_frame_color);
        }
        rt.draw(&rs);
        s.set_texture_rect(res.atlas.rects["ui/invframe"].to_sf());
        s.set_position(pos);
        rt.draw(&s);
        s.set_color(Color::WHITE);
        let Some(item_def) = &game.itemdb.get(slot.id) else {
            continue;
        };
        if let Some(rect) = res.atlas.rects.get(&item_def.graphic_name) {
            let mut rect = rect.to_sf();
            rect.width = rect.width.min(i32::from(TILE_SIZE));
            rect.height = rect.height.min(i32::from(TILE_SIZE));
            s.set_texture_rect(rect);
            s.set_position(Vector2f::from(pos).scv_off(item_def.draw_off));
            rt.draw(&s);
            if item_def.consumable {
                text.set_position(Vector2f::from(pos).scv_off(ScreenVec { x: 2, y: 22 }));
                text.set_string(&slot.qty.to_string());
                rt.draw(&text);
            }
        } else {
            log::error!("Missing rect for item {}", item_def.name);
        }
    }
    text.set_outline_thickness(2.0);
    text.set_position((0., 0.));
    text.set_character_size(18);
    match game
        .ecw
        .query_one_mut::<(&mut MovingEnt, Option<&Health>)>(game.controlled_en)
    {
        Ok((mov, health)) => {
            text.set_string(&format!("Depth: {}", mov.depth_disp()));
            if let Some(health) = health {
                rt.draw(&text);
                text.set_string(&format!("Health: {:.0}/{:.0}", health.current, health.max));
                text.set_position((0., 24.));
            }
        }
        Err(_) => text.set_string("Not controlling anything"),
    }
    if debug.freecam {
        text.set_string(&format!(
            "Mouse depth: {}",
            LengthDisp(debug.mouse_wpos.depth() as f32)
        ));
    }
    rt.draw(&text);
    text.set_position((0., 48.));
    let (hh, mm) = ticks_hm(game.world.ticks);
    text.set_string(&format!("Time: {hh:02}:{mm:02}"));
    rt.draw(&text);
    if game.player_en == hecs::Entity::DANGLING {
        text.set_character_size(30);
        text.set_position((200., 200.));
        text.set_string("YOU ARE DEAD");
        text.set_fill_color(Color::RED);
        rt.draw(&text);
        text.move_((0., 32.));
        text.set_fill_color(Color::YELLOW);
        text.set_string(&format!(
            "Respawning in: {}",
            game.respawn_timer / u32::from(FPS_TARGET)
        ));
        rt.draw(&text);
    }
    if game.ui.menu.open {
        draw_menu(game, rt, res);
    }
}

fn draw_menu(game: &mut GameState, rt: &mut RenderTexture, res: &Res) {
    let Some(list) = game.ui.menu.stack.last() else {
        log::warn!("Trying to draw empty menu");
        return;
    };
    let (x, y, w, h) = (100., 100., 250., 300.);
    let mut rs = RectangleShape::from_rect(Rect::new(x, y, w, h));
    rs.set_fill_color(Color::rgba(20, 20, 40, 120));
    rt.draw(&rs);
    let mut txt = Text::new("Mantle Diver", &res.sans_font, 20);
    txt.set_fill_color(Color::rgb(120, 140, 130));
    txt.set_position((x, y));
    rt.draw(&txt);
    let mut y_offset = 40.0;
    for (i, item) in list.iter().enumerate() {
        if i == game.ui.menu.cursor {
            txt.set_fill_color(game.ui.menu.sel_color);
        } else {
            txt.set_fill_color(Color::WHITE);
        }
        txt.set_position((x, y + y_offset));
        txt.set_string(&item.text);
        rt.draw(&txt);
        y_offset += 22.0;
    }
}

pub fn draw_inventory(game: &mut GameState, cfg: &Config, rt: &mut RenderTexture, res: &Res) {
    let rt_res = rt.res();
    let rect = Inventory::screen_rect(rt_res);
    let sf_rect = rect.into_sf();
    let mut rs = RectangleShape::from_rect(sf_rect);
    let mut s = Sprite::with_texture(&res.atlas.tex);
    rs.set_fill_color(Color::TRANSPARENT);
    rs.set_outline_color(Color::YELLOW);
    rs.set_outline_thickness(2.0);
    rt.draw(&rs);
    let mut text = Text::new("Inventory", &res.sans_font, 20);
    text.set_position((f32::from(rect.x), f32::from(rect.y)));
    rt.draw(&text);
    let inv_bg_color = cfg.ui.inv_bg_color.to_sf();
    let inv_frame_highlight = cfg.ui.inv_frame_highlight.to_sf();
    let inv_frame_color = cfg.ui.inv_frame_color.to_sf();
    rs.set_size((36., 36.));
    for (i, slot) in game.inventory.slots.iter().enumerate() {
        let x = sf_rect.left + ((i * 44) as f32 + 8.0);
        let pos = (x, (sf_rect.top + 48.));
        rs.set_position((pos.0 + 2., pos.1 + 2.));
        rs.set_fill_color(inv_bg_color);
        if i == game.ui.selected_inv_slot {
            s.set_color(inv_frame_highlight);
        } else {
            rs.set_outline_thickness(0.0);
            s.set_color(inv_frame_color);
        }
        rt.draw(&rs);
        s.set_texture_rect(res.atlas.rects["ui/invframe"].to_sf());
        s.set_position(pos);
        rt.draw(&s);
        s.set_color(Color::WHITE);
        let Some(item_def) = &game.itemdb.get(slot.id) else {
            continue;
        };
        if let Some(rect) = res.atlas.rects.get(&item_def.graphic_name) {
            let mut rect = rect.to_sf();
            rect.width = rect.width.min(i32::from(TILE_SIZE));
            rect.height = rect.height.min(i32::from(TILE_SIZE));
            s.set_texture_rect(rect);
            s.set_position(Vector2f::from(pos).scv_off(item_def.draw_off));
            rt.draw(&s);
            if item_def.consumable {
                text.set_position(Vector2f::from(pos).scv_off(ScreenVec { x: 2, y: 22 }));
                text.set_string(&slot.qty.to_string());
                rt.draw(&text);
            }
        } else {
            log::error!("Missing rect for item {}", item_def.name);
        }
    }
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

pub(crate) fn draw_debug_overlay(rt: &mut RenderTexture, game: &mut GameState) {
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
    });
}
