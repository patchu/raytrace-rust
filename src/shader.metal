// src/shader.metal
#include <metal_stdlib>
using namespace metal;

kernel void compute_gradient(
    device float4* output_buffer [[buffer(0)]],
    uint2 thread_position_in_grid [[thread_position_in_grid]]
) {
    uint width = 1280;
    uint height = 720;

    float red = float(thread_position_in_grid.x) / float(width);
    float green = float(thread_position_in_grid.y) / float(height);
    float blue = 0.2;

    uint index = thread_position_in_grid.y * width + thread_position_in_grid.x;

    output_buffer[index] = float4(red, green, blue, 1.0);
}