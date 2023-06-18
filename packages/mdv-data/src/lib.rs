use ron::{extensions::Extensions, ser::PrettyConfig};

/// Ron pretty configuration all data files use
pub fn ron_pretty_cfg() -> PrettyConfig {
    PrettyConfig::default()
        .enumerate_arrays(true)
        .struct_names(true)
        .extensions(Extensions::IMPLICIT_SOME | Extensions::UNWRAP_NEWTYPES)
}
