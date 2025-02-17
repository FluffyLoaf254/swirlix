# Swirlix

## About

A simple, performant voxel sculpting tool made with Rust.

## Status

I've just started this project, and a lot of features are missing or unoptimized.

## Implementation

- Sparse Voxel Octree data structure
- Ray Marching for depth and color
- Normal map generation from depth
- Physically-Based Rendering shader

## Try it Out

To run the project, first make sure you have Rust installed.

Then, clone the project and change into that directory:

```bash
clone git@github.com:FluffyLoaf254/swirlix.git
cd swirlix
```

Then, you can run the project:

```bash
cargo run
```

To run the tests:

```bash
cargo test --workspace
```

## Guide

Left click will add voxels under the cursor, while right click deletes voxels.

Pressing "S" will switch to the square brush and pressing "R" will switch back to the round brush.
