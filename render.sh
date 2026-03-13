#!/bin/bash

# Default: render the three shinys scene (3 bouncing reflective spheres)
cargo run --release

# Other scenes:
# cargo run --release --bin hot_wheels          # Orange Hot Wheels track on wood table
# cargo run --release --bin metal_gradient --features metal  # Metal GPU gradient (macOS only)

# Convert frames to video:
# ffmpeg -framerate 30 -i output/frame_%03d.png -c:v libx264 -pix_fmt yuv420p out.mp4
