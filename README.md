
# MapGen 2D

[![Crates.io](https://img.shields.io/crates/v/mapgen_2d)](https://crates.io/crates/mapgen_2d)
[![docs](https://docs.rs/mapgen_2d/badge.svg)](https://docs.rs/mapgen_2d/)

Utilities for 2d tilemap generation.

## Features

## Voronoi cell computation

Generate voronoi cells in a 2d array, useful e.g. for defining biomes.

![voronoi](images/voronoi.png)

* Optional borders
* Borders are optionally curved (wider near nodes)
* Optional lloyd steps for equalizing cell shape
* See [examples/voronoi.rs](examples/voronoi.rs)

## "Colored" noise

Generate 2d noise with a certain frequency distribution ("color", eg red noise),
useful for heightmaps. Tileable.

![noise](images/noise600.png)

See [examples/noise.rs](examples/noise.rs)

## Wave Function Collapse

Simple "Wave Function Collapse" implementation

![wfc](images/wfc.png)

See [examples/wfc.rs](examples/wfc.rs)

## Coordinate conversions to/from `glam`s `UVec2`

Makes working with coordinates (eg as used in bevy) on the one side and array indices on the other
more bearable.

```rust
let a = Array2::zeros((10, 10));

assert!((3, 4).as_uvec2() == uvec2(3, 4));

let pos = uvec2(3, 4);
let v: f32 = a[pos.as_index2()];
```

## 2d array neighborhood

Utilities for obtaining neighboring tiles (eg as iterator) to a given position.
Will incorporate array boundaries.

```rust
let np = NeighborPositions::new(uvec2(10, 10), ivec2(-1, 8), euclidean, 3);

for pos in np.iter_metric(euclidean) {
	// ...
}
```

Optionally also with holding a reference to your map data:

```rust
let a = Array2::zeros((10, 10));
let np = Neighborhood::new(&a, ivec2(-1, 8), 3);

for (pos, elem) in np.iter_with_positions() {
	// ...
}
```

## Rectangle and Region datatypes

`Rect` is useful for woring with rectangular subsets of arrays
without the need to hold a reference to the underlying array data (unlike a slice).

```rust
let r = Rect::around(uvec2(6, 7), 3);
assert!(r.top_left == uvec2(3, 4));
assert!(r.bottom_right == uvec2(9, 10));

for pos in r.iter_indices() {
	// ...
}
```

For more arbitrary-shaped regions there is `Region` which is defined by a bounding rectangle
and some external array that will be interpreted as a mask.
This is particularly used in `Voronoi` for returning the various generated cells.

```rust
// Define a 6x6 bounding box,
// our region will be fully contained in this
let bounding_box = Rect::from_corners(uvec2(2, 2), uvec2(7, 7));
// The actual region is defined by the bounding box and a `reference` value
// which allows us to identify tiles that belong to the region
let region = Region::new(bounding_box, 1);

// Define a mask for our region
let a = Array2::zeros((10, 10));
a[[3, 3]] = 1_u32;
a[[3, 4]] = 1;
a[[4, 3]] = 1;

// Together with the mask we can iterate
for pos in region.iter_indices(a) {
	// Will iterate over (3, 3), (3, 4) and (4, 3)
}
```

