use {
    super::{for_each_tile_on_screen, GameState, LightSource, SfVec2fExt},
    crate::{
        debug::DebugState,
        graphics::ScreenVec,
        inventory::ItemId,
        math::{smoothwave, WorldPos, TILE_SIZE},
        player::{FacingDir, MovingEnt, PlayerQuery},
        res::Res,
    },
    gamedebug_core::imm_dbg,
    sfml::{
        graphics::{
            Color, Rect, RectangleShape, RenderTarget, RenderTexture, Shape, Sprite, Text,
            Transformable,
        },
        system::Vector2f,
    },
};

trait SpriteExt {
    fn fit_to_size(&mut self, size: Vector2f);
}

impl SpriteExt for Sprite<'_> {
    fn fit_to_size(&mut self, size: Vector2f) {
        let Some(tex) = self.texture() else {
            return
        };
        let self_size: Vector2f = tex.size().as_other();
        self.set_scale((size.x / self_size.x, size.y / self_size.y));
    }
}

pub(crate) fn draw_world(game: &mut GameState, rt: &mut RenderTexture, res: &mut Res) {
    let mut s = Sprite::with_texture(&res.forest_bg);
    s.fit_to_size(rt.size().as_other());
    rt.draw(&s);
    game.light_sources.clear();
    let mut s = Sprite::with_texture(&res.atlas.tex);
    for_each_tile_on_screen(game.camera_offset, rt.size(), |tp, sp| {
        let tile = *game.world.tile_at_mut(tp);
        let spr_pos = sp.to_sf_vec();
        s.set_rotation(0.);
        s.set_scale((1.0, 1.0));
        if !tile.bg.empty() {
            let def = &game.tile_db[tile.bg];
            s.set_position(spr_pos.scv_off(def.draw_offs));
            s.set_texture_rect(def.tex_rect.to_sf());
            rt.draw(&s);
        }
        if !tile.mid.empty() {
            let def = &game.tile_db[tile.mid];
            s.set_position(spr_pos.scv_off(def.draw_offs));
            let mut rect = def.tex_rect.to_sf();
            // Tile blending prototype code
            let rect = if !def.neigh_aware {
                rect
            } else {
                rect.width = TILE_SIZE as i32;
                rect.height = TILE_SIZE as i32;
                let left = game.world.tile_at_mut(tp.x_off(-1)).mid == tile.mid;
                let right = game.world.tile_at_mut(tp.x_off(1)).mid == tile.mid;
                let above = game.world.tile_at_mut(tp.y_off(-1)).mid == tile.mid;
                let below = game.world.tile_at_mut(tp.y_off(1)).mid == tile.mid;
                adjust_blend_rect(&mut rect, left, right, above, below);
                rect
            };
            if let Some(state) = game.transient_block_state.get(&tp) {
                s.set_rotation(state.rot);
                s.set_scale((state.scale, state.scale));
            }
            s.set_texture_rect(rect);
            if let Some(light) = game.tile_db[tile.mid].light {
                let pos = ScreenVec {
                    x: sp.x + light.x,
                    y: sp.y + light.y,
                };
                game.light_sources.push(LightSource { pos });
            }
            rt.draw(&s);
        }
        if !tile.fg.empty() {
            let def = &game.tile_db[tile.fg];
            s.set_position(spr_pos.scv_off(def.draw_offs));
            s.set_texture_rect(def.tex_rect.to_sf());
            rt.draw(&s);
        }
    });
}
pub fn draw_entities(game: &mut GameState, rt: &mut RenderTexture, res: &Res, debug: &DebugState) {
    let mut s = Sprite::with_texture(&res.atlas.tex);
    s.set_origin((16., 16.));
    for (_en, (id, mov)) in game.ecw.query_mut::<(&ItemId, &MovingEnt)>() {
        let pos = mov.mob.en.pos;
        let item_def = &game.itemdb.get(*id).unwrap();
        imm_dbg!(&item_def.name);
        imm_dbg!(&item_def.graphic_name);
        s.set_position((
            (pos.x - game.camera_offset.x as i32) as f32,
            (pos.y - game.camera_offset.y as i32) as f32,
        ));
        imm_dbg!(item_def.tex_rect);
        s.set_texture_rect(item_def.tex_rect.to_sf());
        rt.draw(&s);
    }
    draw_player(game, rt, debug, res);
}

fn draw_player(game: &mut GameState, rt: &mut RenderTexture, debug: &DebugState, res: &Res) {
    draw_player_sprites(game, rt, res);
    if debug.player_bb {
        draw_player_bb(game, rt);
    }
}

fn draw_player_sprites(game: &mut GameState, rt: &mut RenderTexture, res: &Res) {
    let Some((_en, plr)) = game.ecw.query_mut::<PlayerQuery>().into_iter().next() else {
        log::error!("Player query failed");
        return;
    };
    let mut s = Sprite::with_texture(&res.atlas.tex);
    let (x, y, _w, _h) = plr.mov.mob.en.xywh();
    let (base_x, base_y) = (
        (x - game.camera_offset.x as i32) as f32,
        (y - game.camera_offset.y as i32) as f32,
    );
    let (
        mut head_x,
        mut eye_x,
        mut hair_x,
        mut torso_x,
        mut legs_x,
        mut tool_x,
        mut pants_x,
        mut shirt_x,
        mut shoes_x,
    ) = (0.0, 4.0, -4.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    match plr.dat.facing_dir {
        FacingDir::Left => {
            if let Some(offs) = game.char_db.graphic_offsets.get("char/head1") {
                head_x = offs.left.x as f32;
            }
            if let Some(offs) = game.char_db.graphic_offsets.get("char/eye1") {
                eye_x = offs.left.x as f32;
            }
            if let Some(offs) = game.char_db.graphic_offsets.get("char/hair1") {
                hair_x = offs.left.x as f32;
            }
            if let Some(offs) = game.char_db.graphic_offsets.get("char/torso1") {
                torso_x = offs.left.x as f32;
            }
            if let Some(offs) = game.char_db.graphic_offsets.get("char/legs1") {
                legs_x = offs.left.x as f32;
            }
            if let Some(offs) = game.char_db.graphic_offsets.get("char/pants1") {
                pants_x = offs.left.x as f32;
            }
            if let Some(offs) = game.char_db.graphic_offsets.get("char/shirt1") {
                shirt_x = offs.left.x as f32;
            }
            if let Some(offs) = game.char_db.graphic_offsets.get("char/shoes1") {
                shoes_x = offs.left.x as f32;
            }
            if let Some(offs) = game.char_db.graphic_offsets.get("items/woodpick") {
                tool_x = offs.left.x as f32;
            }
        }
        FacingDir::Right => {
            s.set_scale((-1.0, 1.0));
            if let Some(offs) = game.char_db.graphic_offsets.get("char/head1") {
                head_x = offs.right.x as f32;
            }
            if let Some(offs) = game.char_db.graphic_offsets.get("char/eye1") {
                eye_x = offs.right.x as f32;
            }
            if let Some(offs) = game.char_db.graphic_offsets.get("char/hair1") {
                hair_x = offs.right.x as f32;
            }
            if let Some(offs) = game.char_db.graphic_offsets.get("char/torso1") {
                torso_x = offs.right.x as f32;
            }
            if let Some(offs) = game.char_db.graphic_offsets.get("char/legs1") {
                legs_x = offs.right.x as f32;
            }
            if let Some(offs) = game.char_db.graphic_offsets.get("char/pants1") {
                pants_x = offs.right.x as f32;
            }
            if let Some(offs) = game.char_db.graphic_offsets.get("char/shirt1") {
                shirt_x = offs.right.x as f32;
            }
            if let Some(offs) = game.char_db.graphic_offsets.get("char/shoes1") {
                shoes_x = offs.right.x as f32;
            }
            if let Some(offs) = game.char_db.graphic_offsets.get("items/woodpick") {
                tool_x = offs.right.x as f32;
            }
        }
    }
    s.set_color(plr.dat.skin_color);
    // Head
    s.set_texture_rect(res.atlas.rects["char/head1"].to_sf());
    s.set_position((base_x + head_x, base_y));
    rt.draw(&s);
    // Eye
    s.set_color(plr.dat.eye_color);
    s.set_texture_rect(res.atlas.rects["char/eye1"].to_sf());
    s.set_position((base_x + eye_x, base_y + 9.));
    rt.draw(&s);
    // Hair
    s.set_color(plr.dat.hair_color);
    s.set_texture_rect(res.atlas.rects["char/hair1"].to_sf());
    s.set_position((base_x + hair_x, base_y - 4.));
    rt.draw(&s);
    s.set_color(plr.dat.skin_color);
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
    s.set_color(plr.dat.shirt_color);
    rt.draw(&s);
    // Pants
    s.set_texture_rect(res.atlas.rects["char/pants1"].to_sf());
    s.set_position((base_x + pants_x, base_y + 64.0));
    s.set_color(plr.dat.pants_color);
    rt.draw(&s);
    // Shoes
    s.set_texture_rect(res.atlas.rects["char/shoes1"].to_sf());
    s.set_position((base_x + shoes_x, base_y + 87.0));
    s.set_color(plr.dat.shoes_color);
    rt.draw(&s);
    // Tool
    s.set_color(Color::WHITE);
    s.set_texture_rect(res.atlas.rects["items/woodpick"].to_sf());
    s.set_position((base_x + tool_x, base_y + 30.0));
    rt.draw(&s);
}

fn draw_player_bb(game: &mut GameState, rt: &mut RenderTexture) {
    let Some((_en, plr)) = game.ecw.query_mut::<PlayerQuery>().into_iter().next() else {
        return;
    };
    let (x, y, w, h) = plr.mov.mob.en.xywh();
    let mut rect_sh = RectangleShape::new();
    rect_sh.set_fill_color(Color::rgba(255, 255, 255, 96));
    rect_sh.set_position((
        (x - game.camera_offset.x as i32) as f32,
        (y - game.camera_offset.y as i32) as f32,
    ));
    rect_sh.set_size((w as f32, h as f32));
    rt.draw(&rect_sh);
    rect_sh.set_size((2., 2.));
    rect_sh.set_fill_color(Color::RED);
    rect_sh.set_position((
        (plr.mov.mob.en.pos.x - game.camera_offset.x as i32) as f32,
        (plr.mov.mob.en.pos.y - game.camera_offset.y as i32) as f32,
    ));
    rt.draw(&rect_sh);
}

pub fn draw_ui(game: &mut GameState, rt: &mut RenderTexture, res: &Res, ui_dims: Vector2f) {
    let mut s = Sprite::with_texture(&res.atlas.tex);
    let mut rs = RectangleShape::from_rect(Rect::new(0., 0., 40., 40.));
    let mut text = Text::new("", &res.sans_font, 14);
    if let Some(def) = game
        .itemdb
        .get(game.inventory.slots[game.selected_inv_slot].id)
    {
        text.set_outline_thickness(1.0);
        text.set_outline_color(Color::BLACK);
        text.set_position((4., ui_dims.y - 70.0));
        text.set_string(&def.name);
        rt.draw(&text);
        text.set_outline_thickness(0.0);
    }
    rs.set_outline_color(Color::YELLOW);
    rs.set_outline_thickness(1.0);
    rs.set_fill_color(Color::TRANSPARENT);
    for (i, slot) in game.inventory.slots.iter().enumerate() {
        s.set_texture_rect(res.atlas.rects["ui/invframe"].to_sf());
        let pos = ((i * 44) as f32 + 8.0, (ui_dims.y - 48.));
        s.set_position(pos);
        rt.draw(&s);
        if i == game.selected_inv_slot {
            rs.set_position(pos);
            rt.draw(&rs);
        }
        let Some(item_def) = &game.itemdb.get(slot.id) else {
                continue;
            };
        if let Some(rect) = res.atlas.rects.get(&item_def.graphic_name) {
            let mut rect = rect.to_sf();
            rect.width = rect.width.min(TILE_SIZE as i32);
            rect.height = rect.height.min(TILE_SIZE as i32);
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
    rs.set_position((0., 0.));
    rs.set_fill_color(Color::rgba(0, 0, 0, 128));
    rs.set_size((160., 22.));
    rs.set_outline_thickness(0.);
    rt.draw(&rs);
    text.set_position((0., 0.));
    if let Some((_en, plr)) = game.ecw.query_mut::<PlayerQuery>().into_iter().next() {
        text.set_string(&format!("Depth: {}", plr.mov.depth_disp()));
    };
    text.set_character_size(18);
    rt.draw(&text);
    if game.menu.open {
        draw_menu(game, rt, res);
    }
}

fn draw_menu(game: &mut GameState, rt: &mut RenderTexture, res: &Res) {
    let Some(list) = game.menu.stack.last() else {
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
        if i == game.menu.cursor {
            txt.set_fill_color(game.menu.sel_color);
        } else {
            txt.set_fill_color(Color::WHITE);
        }
        txt.set_position((x, y + y_offset));
        txt.set_string(&item.text);
        rt.draw(&txt);
        y_offset += 22.0;
    }
}

pub(crate) fn light_pass(game: &mut GameState, lightmap: &mut RenderTexture, res: &Res) {
    let how_much_below_surface = game.camera_offset.y.saturating_sub(WorldPos::SURFACE);
    let light_dropoff = (how_much_below_surface / 8).min(255) as u8;
    let daylight = 255u8;
    game.ambient_light = daylight.saturating_sub(light_dropoff);
    // Clear light map
    // You can clear to a brighter color to increase ambient light level
    lightmap.clear(Color::rgba(
        game.ambient_light,
        game.ambient_light,
        game.ambient_light,
        255,
    ));
    let mut s = Sprite::with_texture(&res.atlas.tex);
    s.set_texture_rect(res.atlas.rects["light/1"].to_sf());
    for ls in &game.light_sources {
        let flicker = smoothwave(game.world.ticks, 40) as f32 / 64.0;
        s.set_scale((4. + flicker, 4. + flicker));
        s.set_origin((128., 128.));
        s.set_position((ls.pos.x.into(), ls.pos.y.into()));
        lightmap.draw(&s);
    }
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
    rect.left += x * TILE_SIZE as i32;
    rect.top += y * TILE_SIZE as i32;
}
