use std::{fmt::Write, path::Path};

fn main() {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    std::fs::write(Path::new(&out_dir).join("data.rs"), gen_data_source()).unwrap();
}

fn id_ify(input: &str) -> String {
    let mut result = input.replace(' ', "_");
    result.make_ascii_uppercase();
    result
}

fn gen_data_source() -> String {
    let mut out = String::new();
    let itemdb = mdv_data::item::ItemDb::load_or_default("data");
    writeln!(
        &mut out,
        "pub mod item {{
                        use mdv_data::item::ItemId;"
    )
    .unwrap();
    for (id, def) in itemdb.iter() {
        writeln!(
            &mut out,
            "pub const {}: ItemId = ItemId({});",
            id_ify(&def.name),
            id.0
        )
        .unwrap();
    }
    writeln!(&mut out, "}}").unwrap();
    out
}
