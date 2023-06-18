use thiserror::Error;

pub mod char;
pub mod item;
pub mod recipe;
pub mod tile;

use {
    ron::{extensions::Extensions, ser::PrettyConfig},
    serde::{Serialize, Serializer},
    std::collections::{BTreeMap, HashMap},
};

/// Ron pretty configuration all data files use
pub fn ron_pretty_cfg() -> PrettyConfig {
    PrettyConfig::default()
        .enumerate_arrays(true)
        .struct_names(true)
        .extensions(Extensions::IMPLICIT_SOME | Extensions::UNWRAP_NEWTYPES)
}

/// Based on https://stackoverflow.com/a/42723390
pub fn ordered_map<S, K: Ord + Serialize, V: Serialize>(
    hm: &HashMap<K, V>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let ordered: BTreeMap<_, _> = hm.iter().collect();
    ordered.serialize(serializer)
}

#[derive(Error, Debug)]
pub enum LoadError {
    #[error("RON error: {0}")]
    Ron(#[from] ron::Error),
    #[error("RON spanned error: {0}")]
    RonSpanned(#[from] ron::de::SpannedError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
