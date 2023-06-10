use {
    super::{Chunk, ChunkPos},
    crate::{
        math::{WorldPos, TILE_SIZE, WORLD_EXTENT_CHUNKS},
        tiles::{BgTileId, FgTileId, MidTileId},
        world::{default_chunk_tiles, CHUNK_EXTENT, CHUNK_N_TILES},
    },
    simdnoise::NoiseBuilder,
};

impl Chunk {
    pub fn gen(pos: ChunkPos, seed: i32) -> Self {
        let mut tiles = default_chunk_tiles();
        let x = u32::from(pos.x) * u32::from(CHUNK_EXTENT);
        let y = u32::from(pos.y) * u32::from(CHUNK_EXTENT);
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
            #[expect(
                clippy::cast_possible_truncation,
                reason = "We aren't iterating through this many tiles"
            )]
            let i = i as u32;
            let y = y + i / u32::from(CHUNK_EXTENT);
            let local_x = i % u32::from(CHUNK_EXTENT);
            let surf = WorldPos::SURFACE / u32::from(TILE_SIZE);
            #[expect(clippy::cast_possible_truncation, reason = "Scaled noise")]
            let ceil = surf.saturating_add_signed(hnoise[local_x as usize] as i32 / 4);
            #[expect(clippy::cast_possible_truncation, reason = "Scaled noise")]
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
            // Dirt level, just a mass of mostly dirt
            let dirt_bottom = surf + 80;
            #[expect(clippy::cast_possible_truncation, reason = "Scaled noise")]
            if y < dirt_bottom.saturating_add_signed(hnoise[local_x as usize] as i32) {
                t.mid = MidTileId::DIRT;
                t.bg = BgTileId::DIRT;
                if y == ceil {
                    //t.fg = FgTileId::GRASS; // Removed for now
                } else if y > ceil + 2 && noise as u32 % 37 == 0 {
                    t.fg = FgTileId::COAL;
                }
                continue;
            }
            // Default "cave level" generation
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
        // Unbreakable buffer zone at the bottom
        if pos.y > WORLD_EXTENT_CHUNKS {
            for b in &mut tiles {
                b.mid = MidTileId::UNBREAKANIUM;
            }
        }
        Self { tiles }
    }
}
