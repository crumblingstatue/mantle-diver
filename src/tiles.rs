use crate::{config::ron_pretty_cfg, math::ScreenVec};

pub mod tiledb_edit_ui;
use {
    crate::{
        item::ItemId,
        math::{IntRect, TILE_SIZE},
        texture_atlas::RectMap,
    },
    serde::{Deserialize, Serialize},
    std::{
        fmt::Debug,
        marker::PhantomData,
        ops::{Index, RangeInclusive},
    },
};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum LayerAccess {
    Bg,
    Mid,
}

#[derive(Serialize, Deserialize)]
pub struct TileId<Layer>(pub u16, #[serde(skip)] PhantomData<Layer>);

impl<Layer> PartialEq for TileId<Layer> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<Layer> Copy for TileId<Layer> {}
impl<Layer> Clone for TileId<Layer> {
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}
impl<Layer> Debug for TileId<Layer> {
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

#[derive(Debug)]
pub enum Bg {}
#[derive(Debug)]
pub enum Mid {}

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
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TileItemDrop {
    pub qty_range: RangeInclusive<u8>,
    pub id: ItemId,
}
pub trait TileLayer {
    /// Definitions specific to this layer
    type SpecificDef;
    const LAYER: LayerAccess;
}

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

impl BgTileId {
    pub const DIRT: Self = Self(1, PhantomData);
    pub const STONE: Self = Self(2, PhantomData);
}

impl MidTileId {
    pub const DIRT: Self = Self(1, PhantomData);
    pub const STONE: Self = Self(2, PhantomData);
    pub const TORCH: Self = Self(3, PhantomData);
    pub const TREE: Self = Self(6, PhantomData);
    pub const SMALL_ROCK: Self = Self(7, PhantomData);
    pub const STICK: Self = Self(8, PhantomData);
    pub const DIRT_COAL: Self = Self(9, PhantomData);
    pub const STONE_COAL: Self = Self(10, PhantomData);
}

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
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MidDef {
    /// Platform behavior: Horizontally passable, vertically passable upwards
    pub platform: bool,
    /// Collision bounding box
    pub bb: Option<TileBb>,
}

impl<Layer: TileLayer> Debug for TileDef<Layer>
where
    Layer::SpecificDef: Debug,
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

impl<Layer: TileLayer> Default for TileDef<Layer>
where
    Layer::SpecificDef: Default + Debug,
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
        }
    }
}

const DEFAULT_TILE_BB: TileBb = TileBb {
    x: 0,
    y: 0,
    w: TILE_SIZE,
    h: TILE_SIZE,
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct TileBb {
    pub x: u8,
    pub y: u8,
    pub w: u8,
    pub h: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TileDb {
    unknown_bg: TileDef<Bg>,
    unknown_mid: TileDef<Mid>,
    bg: Vec<TileDef<Bg>>,
    mid: Vec<TileDef<Mid>>,
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

    pub(crate) fn update_rects(&mut self, rects: &RectMap) {
        update_rect_def(&mut self.unknown_bg, rects);
        update_rect_def(&mut self.unknown_mid, rects);
        update_rect_db(&mut self.bg, rects);
        update_rect_db(&mut self.mid, rects);
    }
}

fn update_rect_db<Layer: TileLayer>(db: &mut Vec<TileDef<Layer>>, rects: &RectMap)
where
    Layer::SpecificDef: Debug,
{
    for def in db {
        update_rect_def(def, rects);
    }
}

fn update_rect_def<Layer: TileLayer>(
    def: &mut TileDef<Layer>,
    rects: &std::collections::HashMap<String, IntRect>,
) where
    Layer::SpecificDef: Debug,
{
    if !def.graphic_name.is_empty() {
        if let Some(rect) = rects.get(def.graphic_name.as_str()) {
            def.tex_rect = *rect;
            log::info!("Updated rect for {}: {:?}", def.graphic_name.as_str(), rect);
        } else {
            log::error!("Missing texture for {:?}", def.graphic_name.as_str());
        }
    } else {
        log::warn!("Empty graphic name!");
    }
}
