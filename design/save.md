Game has different worlds you can generate.
Each world has its own folder.
Each folder has:
    - Player file (player.dat)
    - Subfolder for regions (regions/)

Each region subfolder has:
x.y.rgn files for each chunk, where x and y are the region coordinates.

# Regions
A region stores multiple chunks in a single file for more optimized storage.
Current plan is to store 8 chunks in a region, in other words, 1024*1024 blocks.
Around one million blocks, compressed with zstd compression.
