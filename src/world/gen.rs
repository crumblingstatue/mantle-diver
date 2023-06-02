use {
    super::{Chunk, ChunkPos},
    crate::{
        tiles::{BgTileId, FgTileId, MidTileId},
        world::{default_chunk_tiles, CHUNK_EXTENT, CHUNK_N_TILES},
    },
    simdnoise::NoiseBuilder,
};

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
        // TODO: Take care to generate all chunks with same seed on same world
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
}