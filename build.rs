use std::{fmt::Write, path::Path};

fn main() {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    std::fs::write(Path::new(&out_dir).join("data.rs"), gen_data_source()).unwrap();
}

fn id_ify(input: &str) -> String {
    let mut result = input.replace(' ', "_");
    result = result.replace('/', "_");
    result.make_ascii_uppercase();
    result
}

fn gen_data_source() -> String {
    let mut out = String::new();
    gen_items(&mut out);
    gen_tiles(&mut out);
    out
}

fn gen_items(out: &mut String) {
    let itemdb = mdv_data::item::ItemDb::load_or_default("data");
    writeln!(
        out,
        "pub mod item {{
                        use mdv_data::item::ItemId;"
    )
    .unwrap();
    for (id, def) in itemdb.iter() {
        writeln!(
            out,
            "pub const {}: ItemId = ItemId({});",
            id_ify(&def.name),
            id.0
        )
        .unwrap();
    }
    writeln!(out, "}}").unwrap();
}

fn gen_tiles(out: &mut String) {
    let tiledb = mdv_data::tile::TileDb::load_or_default("data");
    writeln!(out, "pub mod tile {{").unwrap();
    writeln!(
        out,
        "pub mod bg {{use mdv_data::tile::{{TileId, BgTileId}};use core::marker::PhantomData;"
    )
    .unwrap();
    for (i, def) in tiledb.bg.iter().enumerate() {
        writeln!(
            out,
            "pub const {}: BgTileId = TileId({}, PhantomData);",
            id_ify(&def.graphic_name),
            i + 1
        )
        .unwrap();
    }
    writeln!(out, "}}").unwrap();
    writeln!(
        out,
        "pub mod mid {{use mdv_data::tile::{{TileId, MidTileId}};use core::marker::PhantomData;"
    )
    .unwrap();
    for (i, def) in tiledb.mid.iter().enumerate() {
        writeln!(
            out,
            "pub const {}: MidTileId = TileId({}, PhantomData);",
            id_ify(&def.graphic_name),
            i + 1
        )
        .unwrap();
    }
    writeln!(out, "}}").unwrap();
    writeln!(out, "}}").unwrap();
}
