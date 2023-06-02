mod reg_chunk_existence;
mod serialization;

use {
    self::serialization::save_chunk,
    crate::{
        math::WorldPos,
        player::Player,
        tiles::{BgTileId, FgTileId, MidTileId, TileId},
        world::reg_chunk_existence::ExistenceBitset,
    },
    fnv::FnvHashMap,
    simdnoise::NoiseBuilder,
    std::{
        fmt::Debug,
        fs::File,
        io::Seek,
        path::{Path, PathBuf},
    },
};

pub type ChkPosSc = u16;

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct ChunkPos {
    pub x: ChkPosSc,
    pub y: ChkPosSc,
}

impl ChunkPos {
    /// Returns the region this chunk position belongs to
    pub fn region(&self) -> (u8, u8) {
        (
            (self.x / REGION_CHUNK_EXTENT as ChkPosSc) as u8,
            (self.y / REGION_CHUNK_EXTENT as ChkPosSc) as u8,
        )
    }
    /// Returns the local position in the region (0-7)
    pub fn local(&self) -> (u8, u8) {
        (
            (self.x % REGION_CHUNK_EXTENT as ChkPosSc) as u8,
            (self.y % REGION_CHUNK_EXTENT as ChkPosSc) as u8,
        )
    }
}

#[derive(Debug)]
pub struct World {
    /// The currently loaded chunks
    chunks: FnvHashMap<ChunkPos, Chunk>,
    /// This is the number of ticks since the world has started.
    /// In other words, the age of the world.
    pub ticks: u64,
    pub player: Player,
    pub name: String,
    pub path: PathBuf,
    pub seed: i32,
}

impl World {
    pub fn new(spawn_point: WorldPos, name: &str, path: PathBuf, seed: i32) -> Self {
        Self {
            chunks: Default::default(),
            ticks: Default::default(),
            player: Player::new_at(spawn_point),
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
        let chk = self
            .chunks
            .entry(chk)
            .or_insert_with(|| Chunk::load_or_gen(chk, &self.path, self.seed));
        chk.at_mut(local)
    }
    pub fn save(&self) {
        let result = std::fs::create_dir_all(&self.path);
        log::info!("{result:?}");
        self.save_chunks();
    }
    pub fn save_chunks(&self) {
        for (pos, chk) in self.chunks.iter() {
            save_chunk(pos, chk, &self.path);
        }
    }
}

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
const TILE_BYTES: usize = 3 * 2;

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
}

fn chk_pos(tile: TPosSc) -> ChkPosSc {
    (tile / CHUNK_EXTENT as TPosSc) as ChkPosSc
}

#[test]
fn test_chk_pos() {
    assert_eq!(chk_pos(0), 0);
    assert_eq!(chk_pos(1), 0);
    assert_eq!(chk_pos(127), 0);
    assert_eq!(chk_pos(128), 1);
}

fn chunk_local(global: TPosSc) -> ChkLocalTPosSc {
    (global % CHUNK_EXTENT as TPosSc) as ChkLocalTPosSc
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

// Need to support at least 4 million tiles long
pub type TPosSc = u32;

pub const CHUNK_EXTENT: u16 = 128;
const CHUNK_N_TILES: usize = CHUNK_EXTENT as usize * CHUNK_EXTENT as usize;

type ChunkTiles = [Tile; CHUNK_N_TILES];

fn default_chunk_tiles() -> ChunkTiles {
    [Tile {
        bg: TileId::EMPTY,
        mid: TileId::EMPTY,
        fg: TileId::EMPTY,
    }; CHUNK_N_TILES]
}

#[derive(Debug)]
pub struct Chunk {
    tiles: ChunkTiles,
}

impl Chunk {
    pub fn gen(pos: ChunkPos, seed: i32) -> Self {
        let mut tiles = default_chunk_tiles();
        let x = pos.x as u32 * CHUNK_EXTENT as u32;
        let y = pos.y as u32 * CHUNK_EXTENT as u32;
        let noise = NoiseBuilder::gradient_2d_offset(
            x as f32,
            CHUNK_EXTENT as usize,
            y as f32,
            CHUNK_EXTENT as usize,
        )
        .with_seed(seed)
        .generate_scaled(0.0, 1000.0);
        let hnoise = NoiseBuilder::gradient_1d_offset(x as f32, CHUNK_EXTENT as usize)
            .with_seed(seed)
            .generate_scaled(-10., 10.);
        // TODO: TAke care to generate all chunks with same seed on same world
        assert!(noise.len() == CHUNK_N_TILES);
        for (i, (t, noise)) in tiles.iter_mut().zip(noise.into_iter()).enumerate() {
            let y = y + i as u32 / CHUNK_EXTENT as u32;
            let local_x = i as u32 % CHUNK_EXTENT as u32;
            let ceil = 19_968u32.saturating_add_signed(hnoise[local_x as usize] as i32 / 4);
            if y == ceil - 1 {
                if noise as u32 % 19 == 0 {
                    t.mid = MidTileId::TREE;
                } else if noise as u32 % 17 == 0 {
                    t.mid = MidTileId::SMALL_ROCK;
                } else if noise as u32 % 15 == 0 {
                    t.mid = MidTileId::STICK;
                }
            }
            if y < ceil {
                continue;
            }
            if y < 19980u32.saturating_add_signed(hnoise[local_x as usize] as i32) {
                t.mid = MidTileId::DIRT;
                t.bg = BgTileId::DIRT;
                if y == ceil {
                    t.fg = FgTileId::GRASS;
                }
                continue;
            }
            t.bg = BgTileId::STONE;
            if noise < 550. {
                t.mid = MidTileId::STONE;
            }
            if noise < 120. {
                t.mid = MidTileId::DIRT;
                t.bg = BgTileId::DIRT;
            }
            if noise < 40. {
                t.fg = FgTileId::COAL;
            }
        }
        // Unbreakable layer at bottom
        if pos.y > 798 {
            for b in &mut tiles {
                b.mid = MidTileId::UNBREAKANIUM;
            }
        }
        Self { tiles }
    }

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
            assert_eq!(decomp_data.len(), REGION_BYTES);
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
    crate::bitmanip::nth_bit_set(bitset.0, idx as usize)
}

#[derive(Clone, Copy, Debug)]
pub struct Tile {
    /// Background wall behind entities
    pub bg: BgTileId,
    /// The solid wall on the same level as entities
    pub mid: MidTileId,
    /// A layer on top of the mid wall. Usually ores or decorative pieces.
    pub fg: FgTileId,
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
