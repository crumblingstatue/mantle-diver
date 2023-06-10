# Premise

- Evil source at bottom of playable map
- Evil is surging upwards
- Player is the one tasked with going to the bottom and destroying evil source
- Very long way to bottom
- Going deeper = main source progression
- Barriers to going deeper, gotta craft gear, etc. to be able to go deeper
- Tower defense elements, have to craft and set up defensive structures

# Details

## Timing
Timing is tick based.
Every frame is considered a tick.
The game is locked at 60 fps to simplify implementation.

## Scale
Player character is slightly below 1.5 m height.
1 meter = 2 blocks.
1 block = 0.5 meters.
Player character is about 3 blocks high. (slightly less so they fit comfortably)

## Map Size

World is roughly 250,000x250,000 blocks large.
I've done some measurements and I believe it's large enough that it should convey
the scale of digging very deep, which is what the game is about.
Layer below 30 km is the mantle (after which the game is named).
The evil source is at about 35 km.
The final goal is the evil source, so player is expected to mine about 35 km deep to beat the game.
1 meter = two tiles. 35 km = 70,000 tiles deep.
There could be some post-endgame content from 35-40 kms deep.
Very unbreakable bottom or lava lake or whatever is at 80,000 blocks (40 km) deep.

## Biomes
Only a few surface biomes
- At center is forest biome
- On one side:
  - Jungle
  - Desert
- On other side:
  - Longer stretch of forest biome than other side
  - Ice biome

All other biome variety should come from going deeper.
Progressively more difficult variants of each biome as you go down.
30 km is mantle level, at which point no ice or sand or such reaches.
It's all mantle level specific biomes all the way horizontally.

## Number of stuff

- At least 50 ore types + 50 gem/etc. types. Copper/Coal could be an ore-gem/etc. pair for example.
- On average, one new pair every 750 meters, up to 37 kilometers.
  The last stretch of 3 km to the bottom is a "buffer zone" of no new content, just more of the same.
- The rate of new pairs could start out quicker, for example every 300 meters or so,
  to make early-game less boring, but gradually become less frequent, up to a kilometer gap.
- At least 50 stone types
- Maybe background tiles and other stuff, like different break overlay animations


# Multiplayer

Multiplayer adds a lot of complexity, so I'm going to develop the prototype of the game without
any multiplayer support.
But if people like the general idea, they can help create a multiplayer version eventually.

# Crafting

- You need a workstation to be able to craft anything.
  This incentivizes building bases and retreating to them, instead of the "nomad"
  lifetyle many games of this kind allow.

- Most stations don't have inventory items that can be pre-crafted and placed.
  Instead, they have to be constructed on the spot, taking construction time.
  This discourages building makeshift crafting areas in the middle of nowhere.

# Supported platforms
- Windows
- Linux

Other platforms can be supported if community helps support them.
