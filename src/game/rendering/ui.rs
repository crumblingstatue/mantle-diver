use {
    crate::{
        config::Config,
        debug::DebugState,
        game::{ui::Inventory, GameState},
        math::{IntRectExt, FPS_TARGET, TILE_SIZE},
        player::{Health, MovingEnt},
        res::Res,
        sfml::{RectangleShapeExt, RenderTargetExt, ScreenRectSfExt, ScreenVecSfExt, SfVec2fExt},
        stringfmt::LengthDisp,
        time::ticks_hm,
    },
    gamedebug_core::imm_dbg,
    mdv_math::types::ScreenVec,
    sfml::{
        graphics::{
            Color, Rect, RectangleShape, RenderTarget, RenderTexture, Shape, Sprite, Text,
            Transformable,
        },
        system::Vector2f,
    },
};

pub fn draw_ui(
    game: &mut GameState,
    rt: &mut RenderTexture,
    res: &Res,
    cfg: &Config,
    debug: &DebugState,
    mouse_pos: ScreenVec,
    scale: u8,
) {
    if game.ui.inv.open {
        draw_inventory(game, cfg, rt, res, mouse_pos, scale);
    }
    let mut text = Text::new("", &res.sans_font, 14);
    draw_hotbar(game, &mut text, rt, cfg, res);
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

fn draw_hotbar(
    game: &GameState,
    text: &mut Text<'_>,
    rt: &mut RenderTexture,
    cfg: &Config,
    res: &Res,
) {
    let rt_size: Vector2f = rt.size().as_other();
    let mut s = Sprite::with_texture(&res.atlas.tex);
    let mut rs = RectangleShape::from_rect(Rect::new(0., 0., 36., 36.));
    if let Some(def) = game
        .itemdb
        .get(game.inventory.slots[game.ui.selected_inv_slot].id)
    {
        text.set_outline_thickness(1.0);
        text.set_outline_color(Color::BLACK);
        text.set_position((4., rt_size.y - 70.0));
        text.set_string(&def.name);
        rt.draw(&*text);
        text.set_outline_thickness(0.0);
    }
    let inv_frame_color = cfg.ui.inv_frame_color.to_sf();
    let inv_frame_highlight = cfg.ui.inv_frame_highlight.to_sf();
    let inv_bg_color = cfg.ui.inv_bg_color.to_sf();
    for (i, (slot, rect)) in game
        .inventory
        .slots
        .iter()
        .take(10)
        .zip(game.ui.hotbar_rects)
        .enumerate()
    {
        rs.set_screen_rect(rect);
        let (x, y) = (f32::from(rect.x), f32::from(rect.y));
        rs.set_position((x + 2., y + 2.));
        rs.set_fill_color(inv_bg_color);
        if i == game.ui.selected_inv_slot {
            s.set_color(inv_frame_highlight);
        } else {
            rs.set_outline_thickness(0.0);
            s.set_color(inv_frame_color);
        }
        rt.draw(&rs);
        s.set_texture_rect(res.atlas.rects["ui/invframe"].to_sf());
        s.set_position((x, y));
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
            s.set_position(Vector2f::from((x, y)).scv_off(item_def.draw_off));
            rt.draw(&s);
            if item_def.stackable {
                text.set_position(Vector2f::from((x, y)).scv_off(ScreenVec { x: 2, y: 22 }));
                text.set_string(&slot.qty.to_string());
                rt.draw(&*text);
            }
        } else {
            log::error!("Missing rect for item {}", item_def.name);
        }
    }
}

fn draw_menu(game: &GameState, rt: &mut RenderTexture, res: &Res) {
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

fn draw_inventory(
    game: &GameState,
    cfg: &Config,
    rt: &mut RenderTexture,
    res: &Res,
    mouse_pos: ScreenVec,
    scale: u8,
) {
    let rt_res = rt.res();
    let rect = Inventory::screen_rect(rt_res, game.inventory.slots.len());
    let sf_rect = rect.into_sf();
    let mut rs = RectangleShape::from_rect(sf_rect);
    let mut s = Sprite::with_texture(&res.atlas.tex);
    rs.set_fill_color(Color::TRANSPARENT);
    rs.set_outline_thickness(3.0);
    rs.set_outline_color(cfg.ui.inv_frame_color.coffset(50).to_sf());
    rt.draw(&rs);
    rs.set_outline_color(cfg.ui.inv_frame_color.to_sf());
    rs.set_outline_thickness(2.0);
    rt.draw(&rs);
    rs.set_outline_thickness(1.0);
    rs.set_outline_color(cfg.ui.inv_frame_color.coffset(-50).to_sf());
    rs.set_fill_color(cfg.ui.inv_bg_color.to_sf());
    rt.draw(&rs);
    let mut text = Text::new("Inventory", &res.sans_font, 16);
    text.set_outline_thickness(1.0);
    text.set_outline_color(Color::BLACK);
    text.set_position((f32::from(rect.x + 2), f32::from(rect.y)));
    rt.draw(&text);
    text.set_character_size(14);
    let inv_bg_color = cfg.ui.inv_bg_color.to_sf();
    let inv_frame_highlight = cfg.ui.inv_frame_highlight.to_sf();
    let inv_frame_color = cfg.ui.inv_frame_color.to_sf();
    imm_dbg!(game.inventory.slots.len());
    for ((i, slot), srect) in game
        .inventory
        .slots
        .iter()
        .enumerate()
        .zip(&game.ui.inv_rects)
    {
        let pos = srect.sf_position();
        rs.set_screen_rect(srect.offset(1, 1));
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
            if item_def.stackable {
                text.set_position(Vector2f::from(pos).scv_off(ScreenVec { x: 2, y: 22 }));
                text.set_string(&slot.qty.to_string());
                rt.draw(&text);
            }
        } else {
            log::error!("Missing rect for item {}", item_def.name);
        }
    }
    if let Some(grabbed) = &game.inventory.grabbed {
        let Some(item_def) = &game.itemdb.get(grabbed.id) else {
            return;
        };
        let pos = mouse_pos.scaled(scale).to_sf_vec2f();
        if let Some(rect) = res.atlas.rects.get(&item_def.graphic_name) {
            let mut rect = rect.to_sf();
            rect.width = rect.width.min(i32::from(TILE_SIZE));
            rect.height = rect.height.min(i32::from(TILE_SIZE));
            s.set_texture_rect(rect);
            s.set_position(Vector2f::from(pos).scv_off(item_def.draw_off));
            rt.draw(&s);
            if item_def.stackable {
                text.set_position(Vector2f::from(pos).scv_off(ScreenVec { x: 2, y: 22 }));
                text.set_string(&grabbed.qty.to_string());
                rt.draw(&text);
            }
        } else {
            log::error!("Missing rect for item {}", item_def.name);
        }
    }
}
