#!/bin/bash
set -e

mkdir -p output

echo "Building three_shinys (release)..."
cargo build --release --bin three_shinys

echo "Rendering frames..."
cargo run --release --bin three_shinys

echo "Converting frames to video..."
ffmpeg -y -framerate 30 -i output/frame_%03d.png -c:v libx264 -pix_fmt yuv420p threeShinys.mp4

echo "Done! Video saved to threeShinys.mp4"

# Other scenes:
# cargo run --release --bin hot_wheels
# cargo run --release --bin metal_gradient --features metal  # macOS only
