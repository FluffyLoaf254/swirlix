# ![swirlix-logo](https://raw.githubusercontent.com/FluffyLoaf254/swirlix/refs/heads/main/desktop/icons/logo-32.svg) Swirlix

## About

A simple, performant voxel sculpting tool made with Rust.

## Status

A lot of features are missing or incomplete.

## Implementation

- Sparse Voxel Octree data structure
- Ray Marching for color and normal
- Simple Blinn-Phong rendering

## Try it Out

To run the project, first make sure you have Rust installed.

Then, clone the project and change into that directory:

```bash
git clone git@github.com:FluffyLoaf254/swirlix.git
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

Left clicking will add voxels under the cursor, while right clicking deletes voxels.

Pressing "S" will switch to the square brush and pressing "R" will switch back to the round brush.

## Contributing

This project is still very early in development, so there will be a lot of breaking changes. If you'd like to contribute, I'd welcome discussion in the issues. Thanks!
