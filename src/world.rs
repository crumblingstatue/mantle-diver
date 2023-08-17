use {
    crate::{
        math::{WorldPos, TILE_SIZE},
        time::HOUR_IN_TICKS,
    },
    mdv_data::tile::{BgTileId, MidTileId, TileId},
};

mod gen;
mod reg_chunk_existence;
mod serialization;

use {
    self::serialization::save_chunk,
    crate::world::reg_chunk_existence::ExistenceBitset,
    std::{
        fmt::Debug,
        fs::File,
        io::Seek,
        path::{Path, PathBuf},
    },
};

/// Invariant: Can't exceed CHK_POS_SC_MAX
pub type ChkPosSc = u16;

/// Each region holds 8 chunks in one direction (8x8).
/// Region index is a byte, so 255 max.
/// 255 * 8 = 2040
pub const CHK_POS_SC_MAX: ChkPosSc = REGION_CHUNK_EXTENT as ChkPosSc * 255;
#[expect(clippy::assertions_on_constants)]
const _: () = assert!(
    CHK_POS_SC_MAX == 2040,
    "Assumption about chunk pos scalar max value broken"
);

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct ChunkPos {
    pub x: ChkPosSc,
    pub y: ChkPosSc,
}

impl ChunkPos {
    /// Returns the region this chunk position belongs to
    #[expect(
        clippy::cast_possible_truncation,
        reason = "Fits into u8. See CHK_POS_SC_MAX."
    )]
    pub fn region(&self) -> (u8, u8) {
        (
            (self.x / ChkPosSc::from(REGION_CHUNK_EXTENT)) as u8,
            (self.y / ChkPosSc::from(REGION_CHUNK_EXTENT)) as u8,
        )
    }
    /// Returns the local position in the region (0-7)
    #[expect(
        clippy::cast_possible_truncation,
        reason = "Fits due to prior modulo arithmetic"
    )]
    pub fn local(&self) -> (u8, u8) {
        (
            (self.x % ChkPosSc::from(REGION_CHUNK_EXTENT)) as u8,
            (self.y % ChkPosSc::from(REGION_CHUNK_EXTENT)) as u8,
        )
    }
}

#[derive(Debug)]
pub struct World {
    /// The currently loaded chunks
    pub chunks: Vec<(ChunkPos, Chunk)>,
    /// This is the number of ticks since the world has started.
    /// In other words, the age of the world.
    pub ticks: u64,
    pub name: String,
    pub path: PathBuf,
    pub seed: i32,
}

impl World {
    pub fn new(name: &str, path: PathBuf, seed: i32) -> Self {
        // Ensure world dir exists, as chunks could be saved at any time during gameplay
        std::fs::create_dir_all(&path).unwrap();
        Self {
            chunks: Default::default(),
            ticks: 8 * HOUR_IN_TICKS,
            name: name.to_string(),
            path,
            seed,
        }
    }
    /// Get mutable access to the tile at `pos`.
    ///
    /// Loads or generates the containing chunk if necessary.
    pub fn tile_at_mut(&mut self, pos: TilePos) -> &mut Tile {
        let (chk, local) = pos.to_chunk_and_local();
        let chk = match self.chunks.iter().position(|(p, _)| *p == chk) {
            Some(idx) => &mut self.chunks[idx].1,
            None => {
                self.chunks
                    .push((chk, Chunk::load_or_gen(chk, &self.path, self.seed)));
                &mut self.chunks.last_mut().unwrap().1
            }
        };
        chk.at_mut(local)
    }
    pub fn save(&self) {
        self.save_chunks();
    }
    pub fn save_chunks(&self) {
        for (pos, chk) in self.chunks.iter() {
            save_chunk(pos, chk, &self.path);
        }
    }
    pub fn remove_old_chunks(&mut self) {
        while self.chunks.len() > MAX_LOADED_CHUNKS {
            let (pos, chk) = self.chunks.remove(0);
            save_chunk(&pos, &chk, &self.path);
        }
    }
}

const MAX_LOADED_CHUNKS: usize = 16;

fn loc_byte_idx_xy(x: u8, y: u8) -> usize {
    loc_byte_idx(loc_idx(y, x))
}

fn loc_byte_idx(loc_idx: u8) -> usize {
    loc_idx as usize * CHUNK_BYTES
}

fn loc_idx(loc_y: u8, loc_x: u8) -> u8 {
    (loc_y * REGION_CHUNK_EXTENT) + loc_x
}

fn format_reg_file_name((x, y): (u8, u8)) -> String {
    format!("{x}.{y}.rgn")
}

const CHUNK_BYTES: usize = CHUNK_N_TILES * TILE_BYTES;
const TILE_BYTES: usize = 2 * 2;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct TilePos {
    pub x: TPosSc,
    pub y: TPosSc,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ChunkLocalTilePos {
    pub x: ChkLocalTPosSc,
    pub y: ChkLocalTPosSc,
}

/// Chunk-local tile position scalar. Supports up to 256 tiles per chunk.
type ChkLocalTPosSc = u8;

impl TilePos {
    pub fn to_chunk_and_local(self) -> (ChunkPos, ChunkLocalTilePos) {
        let chk = ChunkPos {
            x: chk_pos(self.x),
            y: chk_pos(self.y),
        };
        let local = ChunkLocalTilePos {
            x: chunk_local(self.x),
            y: chunk_local(self.y),
        };
        (chk, local)
    }

    pub(crate) fn to_chunk(self) -> ChunkPos {
        self.to_chunk_and_local().0
    }

    pub(crate) fn x_off(&self, off: i32) -> Self {
        Self {
            x: self.x.saturating_add_signed(off),
            y: self.y,
        }
    }

    pub(crate) fn y_off(&self, off: i32) -> TilePos {
        Self {
            x: self.x,
            y: self.y.saturating_add_signed(off),
        }
    }

    pub(crate) fn subtract(&self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    #[expect(dead_code, reason = "Could be useful in the future")]
    pub(crate) fn to_world(self) -> WorldPos {
        WorldPos {
            x: self.x * u32::from(TILE_SIZE),
            y: self.y * u32::from(TILE_SIZE),
        }
    }
    #[expect(
        clippy::cast_possible_wrap,
        reason = "Tile pos doesn't exceed i32::MAX"
    )]
    pub(crate) fn to_s2dc_en_pos(self) -> (i32, i32) {
        (
            self.x as i32 * i32::from(TILE_SIZE),
            self.y as i32 * i32::from(TILE_SIZE),
        )
    }
    /// Vertical surface level.
    /// You can build roughly 21 km high.
    /// This configuration allows for:
    /// - The player being able to reach 20 kms high comfortably
    /// - Reaching 100 kms deep with a comfortable buffer zone until bottom boundary
    pub const SURFACE: TPosSc = 42_000;
}

#[expect(clippy::cast_possible_truncation, reason = "See `CHK_POS_SC_MAX`")]
fn chk_pos(tile: TPosSc) -> ChkPosSc {
    (tile / TPosSc::from(CHUNK_EXTENT)) as ChkPosSc
}

#[test]
fn test_chk_pos() {
    assert_eq!(chk_pos(0), 0);
    assert_eq!(chk_pos(1), 0);
    assert_eq!(chk_pos(127), 0);
    assert_eq!(chk_pos(128), 1);
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "Sound due to modulo with `CHUNK_EXTENT`"
)]
fn chunk_local(global: TPosSc) -> ChkLocalTPosSc {
    (global % TPosSc::from(CHUNK_EXTENT)) as ChkLocalTPosSc
}

#[test]
fn test_chunk_local() {
    assert_eq!(chunk_local(0), 0);
}

#[test]
fn test_to_chunk_and_local() {
    assert_eq!(
        TilePos { x: 0, y: 0 }.to_chunk_and_local(),
        (ChunkPos { x: 0, y: 0 }, ChunkLocalTilePos { x: 0, y: 0 })
    );
    assert_eq!(
        TilePos { x: 1, y: 1 }.to_chunk_and_local(),
        (ChunkPos { x: 0, y: 0 }, ChunkLocalTilePos { x: 1, y: 1 })
    );
}

pub type TPosSc = u32;

#[expect(dead_code)]
pub const TPOS_SC_MAX: TPosSc = CHK_POS_SC_MAX as TPosSc * CHUNK_EXTENT as TPosSc;
#[expect(clippy::assertions_on_constants)]
const _: () = assert!(
    TPOS_SC_MAX == 261_120,
    "Assumption about max tilepos value broken"
);

pub const CHUNK_EXTENT: u16 = 128;
const CHUNK_N_TILES: usize = CHUNK_EXTENT as usize * CHUNK_EXTENT as usize;

type ChunkTiles = [Tile; CHUNK_N_TILES];

fn default_chunk_tiles() -> ChunkTiles {
    [Tile {
        bg: TileId::EMPTY,
        mid: TileId::EMPTY,
    }; CHUNK_N_TILES]
}

#[derive(Debug)]
pub struct Chunk {
    tiles: ChunkTiles,
}

impl Chunk {
    pub fn load_or_gen(chk: ChunkPos, world_path: &Path, seed: i32) -> Chunk {
        log::info!("Loading chunk {chk:?} (reg: {:?})", chk.region());
        let reg_filename = world_path.join(format_reg_file_name(chk.region()));
        if chunk_exists(&reg_filename, chk) {
            log::info!("Chunk exists, loading");
            let mut f = File::open(&reg_filename).unwrap();
            let bitset = ExistenceBitset::read_from_file(&mut f);
            log::info!("Existence bitset: {bitset:?}");
            assert_eq!(f.stream_position().unwrap(), 8);
            let decomp_data = zstd::decode_all(f).unwrap();
            if decomp_data.len() != REGION_BYTES {
                log::error!("Decompressed data length different than REGION_BYTES");
                return Self {
                    tiles: default_chunk_tiles(),
                };
            }
            let local_pos = chk.local();
            Chunk::load_from_region(&decomp_data, local_pos.0, local_pos.1)
        } else {
            log::warn!("Chunk at {:?} doesn't exist, generating.", chk);
            Chunk::gen(chk, seed)
        }
    }

    fn at_mut(&mut self, local: ChunkLocalTilePos) -> &mut Tile {
        &mut self.tiles[CHUNK_EXTENT as usize * local.y as usize + local.x as usize]
    }
}

fn chunk_exists(reg_path: &Path, pos: ChunkPos) -> bool {
    if !Path::new(&reg_path).exists() {
        return false;
    }
    let bitset = ExistenceBitset::read_from_fs(reg_path);
    let local = pos.local();
    let idx = loc_idx(local.1, local.0);
    mdv_math::bitmanip::nth_bit_set(bitset.0, idx as usize)
}

#[derive(Clone, Copy, Debug)]
pub struct Tile {
    /// Background wall behind entities
    pub bg: BgTileId,
    /// The solid wall on the same level as entities
    pub mid: MidTileId,
}

pub const REGION_CHUNK_EXTENT: u8 = 8;
pub const REGION_N_CHUNKS: u8 = REGION_CHUNK_EXTENT * REGION_CHUNK_EXTENT;
/// This is the uncompressed byte length of a region
pub const REGION_BYTES: usize = REGION_N_CHUNKS as usize * CHUNK_BYTES;

#[expect(clippy::assertions_on_constants)]
const _: () = assert!(
    REGION_N_CHUNKS <= 64,
    "A region file uses an existence bitset that's a 64 bit integer"
);
