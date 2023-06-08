# Lighting system

## Requirements
- Ambient light (day/night, zero at cave layer)
- Backwalls block ambient light
- Mid tiles block both ambient and active light

## Implementation

We can use a lightmap that tells what light level each tile should be.
We can use a flood fill algorithm to propagate light from both light sources
and uncovered tiles.
The lightmap itself needs to be as big as how many tiles fit on screen.
This should include partially visible tiles.
