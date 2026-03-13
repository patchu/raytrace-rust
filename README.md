# raytrace

A CPU raytracer written in Rust that renders animated 3D scenes to PNG frames.

## Scenes

- **three_shinys** (default) — 3 bouncing reflective spheres on a checkerboard floor with a sunset sky and procedural clouds. The camera orbits around the scene over 600 frames.
- **hot_wheels** — An orange Hot Wheels racing track on a wood-textured table. The camera flies over the oval track across 120 frames. Uses parallel rendering with rayon.
- **metal_gradient** — An experimental Metal GPU compute shader that renders a color gradient. macOS only.

## Build & Run

```bash
# Build and run the default scene (three_shinys)
cargo run --release

# Run a specific scene
cargo run --release --bin hot_wheels
cargo run --release --bin metal_gradient --features metal  # macOS only

# Or use the helper script
./render.sh
```

Frames are saved to the `output/` directory.

## Converting Frames to Video

```bash
ffmpeg -framerate 30 -i output/frame_%03d.png -c:v libx264 -pix_fmt yuv420p out.mp4
```

## Dependencies

- [nalgebra](https://crates.io/crates/nalgebra) — Linear algebra (vectors, dot products, etc.)
- [image](https://crates.io/crates/image) — PNG encoding
- [rayon](https://crates.io/crates/rayon) — Parallel scanline rendering (used by hot_wheels)
- [indicatif](https://crates.io/crates/indicatif) — Progress bars (used by hot_wheels)
- [metal](https://crates.io/crates/metal) — Apple Metal GPU API (optional, macOS only)

## Platform Notes

The `metal_gradient` scene requires macOS and is gated behind the `metal` Cargo feature. All other scenes are cross-platform.
