use {
    crate::{item::ItemId, ron_pretty_cfg},
    mdv_math::types::{IntRect, ScreenVec},
    serde::{Deserialize, Serialize},
    std::{
        marker::PhantomData,
        ops::{Index, RangeInclusive},
    },
};

#[derive(Serialize, Deserialize)]
pub struct TileDef<Layer: TileLayer> {
    /// Whether the tile emits light, and the light source offset
    pub light: Option<ScreenVec>,
    pub graphic_name: String,
    #[serde(skip)]
    pub tex_rect: IntRect,
    pub layer: Layer::SpecificDef,
    /// Neighbour aware tile, 4x4 tileset, select based on
    /// neighbouring tiles
    pub neigh_aware: bool,
    /// What tiles this tile can blend with
    #[serde(default)]
    pub blend_list: Vec<u16>,
    pub draw_offs: ScreenVec,
    /// Starting (and maximum) health of a tile.
    /// If it reaches zero, the tile gets destroyed.
    pub health: f32,
    #[serde(default)]
    pub hit_sound: Option<String>,
    #[serde(default)]
    pub item_drop: Option<TileItemDrop>,
    /// Whether the tile can be "uprooted" by interacting with it (sticks/stones/plants, etc.)
    #[serde(default)]
    pub uprootable: bool,
}
impl TileDef<Mid> {
    /// Whether this tile is solid/impassable for entities
    pub fn is_impassable(&self) -> bool {
        self.layer.bb.is_some() && !self.layer.platform
    }
}

impl<Layer: TileLayer> std::fmt::Debug for TileDef<Layer>
where
    Layer::SpecificDef: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TileDef")
            .field("light", &self.light)
            .field("graphic_name", &self.graphic_name)
            .field("tex_rect", &self.tex_rect)
            .field("layer", &self.layer)
            .finish()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TileItemDrop {
    pub qty_range: RangeInclusive<u8>,
    pub id: ItemId,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum LayerAccess {
    Bg,
    Mid,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct TileBb {
    pub x: u8,
    pub y: u8,
    pub w: u8,
    pub h: u8,
}

pub trait TileLayer {
    /// Definitions specific to this layer
    type SpecificDef;
    const LAYER: LayerAccess;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TileDb {
    pub unknown_bg: TileDef<Bg>,
    pub unknown_mid: TileDef<Mid>,
    pub bg: Vec<TileDef<Bg>>,
    pub mid: Vec<TileDef<Mid>>,
}

impl Default for TileDb {
    fn default() -> Self {
        Self {
            unknown_bg: Bg::unknown_def(),
            unknown_mid: Mid::unknown_def(),
            bg: vec![],
            mid: vec![],
        }
    }
}

#[derive(Debug)]
pub enum Bg {}
#[derive(Debug)]
pub enum Mid {}

impl TileLayer for Bg {
    type SpecificDef = ();
    const LAYER: LayerAccess = LayerAccess::Bg;
}

impl TileLayer for Mid {
    type SpecificDef = MidDef;
    const LAYER: LayerAccess = LayerAccess::Mid;
}

pub type BgTileId = TileId<Bg>;
pub type MidTileId = TileId<Mid>;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MidDef {
    /// Platform behavior: Horizontally passable, vertically passable upwards
    pub platform: bool,
    /// Collision bounding box
    pub bb: Option<TileBb>,
}

#[derive(Serialize, Deserialize)]
pub struct TileId<Layer>(pub u16, #[serde(skip)] pub PhantomData<Layer>);

impl Bg {
    pub fn unknown_def() -> TileDef<Self> {
        TileDef {
            light: Some(ScreenVec { x: 0, y: 0 }),
            graphic_name: String::from("tiles/unknown_bg"),
            tex_rect: IntRect::default(),
            draw_offs: ScreenVec::default(),
            layer: (),
            neigh_aware: false,
            health: 10.0,
            hit_sound: None,
            item_drop: None,
            blend_list: vec![],
            uprootable: false,
        }
    }
}

impl Mid {
    pub fn unknown_def() -> TileDef<Self> {
        TileDef {
            light: Some(ScreenVec { x: 0, y: 0 }),
            graphic_name: String::from("tiles/unknown_mid"),
            tex_rect: IntRect::default(),
            draw_offs: ScreenVec::default(),
            layer: MidDef {
                platform: true,
                bb: Some(TileBb {
                    x: 0,
                    y: 0,
                    w: TILE_SIZE,
                    h: TILE_SIZE,
                }),
            },
            neigh_aware: false,
            health: 10.0,
            hit_sound: None,
            item_drop: None,
            blend_list: vec![],
            uprootable: false,
        }
    }
}

/// FIXME: This is a duplicated constant (one exists in mantle-diver crate)
pub const TILE_SIZE: u8 = 32;

impl<Layer> PartialEq for TileId<Layer> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<Layer> Copy for TileId<Layer> {}
impl<Layer> Clone for TileId<Layer> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<Layer> std::fmt::Debug for TileId<Layer> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TileId").field(&self.0).finish()
    }
}

impl<Layer> TileId<Layer> {
    pub fn from_idx(idx: u16) -> Self {
        Self(idx, PhantomData)
    }
    pub fn empty(&self) -> bool {
        self.0 == 0
    }
    pub const EMPTY: Self = Self(0, PhantomData);
}

impl<Layer: TileLayer> Default for TileDef<Layer>
where
    Layer::SpecificDef: Default + std::fmt::Debug,
{
    fn default() -> Self {
        Self {
            light: Default::default(),
            graphic_name: Default::default(),
            tex_rect: Default::default(),
            draw_offs: ScreenVec::default(),
            layer: Layer::SpecificDef::default(),
            neigh_aware: false,
            health: 10.0,
            hit_sound: None,
            item_drop: None,
            blend_list: vec![],
            uprootable: false,
        }
    }
}

impl Index<BgTileId> for TileDb {
    type Output = TileDef<Bg>;

    fn index(&self, index: BgTileId) -> &Self::Output {
        assert!(index.0 != 0);
        self.bg
            .get(index.0 as usize - 1)
            .unwrap_or(&self.unknown_bg)
    }
}

impl Index<MidTileId> for TileDb {
    type Output = TileDef<Mid>;

    fn index(&self, index: MidTileId) -> &Self::Output {
        assert!(index.0 != 0);
        self.mid
            .get(index.0 as usize - 1)
            .unwrap_or(&self.unknown_mid)
    }
}

const PATH: &str = "tiles.ron";

impl TileDb {
    pub fn load_or_default(data_path: &str) -> Self {
        match std::fs::read_to_string(format!("{data_path}/{PATH}")) {
            Ok(data) => match ron::from_str(&data) {
                Ok(db) => db,
                Err(e) => {
                    log::warn!("Failed to load tile database: {e}\nCreating default.");
                    Default::default()
                }
            },
            Err(e) => {
                log::warn!("Failed to load tile database: {e}\nCreating default.");
                Default::default()
            }
        }
    }
    pub fn try_save(&self, data_path: &str) {
        match ron::ser::to_string_pretty(self, ron_pretty_cfg()) {
            Ok(str) => match std::fs::write(format!("{data_path}/{PATH}"), str) {
                Ok(()) => {}
                Err(e) => log::warn!("Failed to save tile db: {e}"),
            },
            Err(e) => log::warn!("Failed to save tile db: {e}"),
        }
    }
}
