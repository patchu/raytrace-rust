// src/metal_gradient.rs
use image::{Rgba, RgbaImage};
use metal::*;
use std::mem;

fn main() {
    // --- 1. Setup Metal Device and Command Queue ---
    let device = Device::system_default().expect("No Metal device found");
    let queue = device.new_command_queue();

    // --- 2. Load the Pre-compiled Shader Library ---
    // This macro combination should now work correctly.
    // It finds the OUT_DIR env var set by Cargo and includes the file from there.
    const SHADER_DATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/shader.metallib"));
    let library = device.new_library_with_data(SHADER_DATA).unwrap();
    let compute_fn = library.get_function("compute_gradient", None).unwrap();

    // --- 3. Create the Compute Pipeline ---
    let pipeline_state = device.new_compute_pipeline_state_with_function(&compute_fn).unwrap();

    // --- 4. Create Buffers on the GPU ---
    let width: u64 = 1280;
    let height: u64 = 720;

    let buffer_size = width * height * 4 * mem::size_of::<f32>() as u64;
    let output_buffer = device.new_buffer(buffer_size, MTLResourceOptions::StorageModeShared);

    // --- 5. Encode and Dispatch the Kernel ---
    let command_buffer = queue.new_command_buffer();
    let compute_encoder = command_buffer.new_compute_command_encoder();

    compute_encoder.set_compute_pipeline_state(&pipeline_state);
    compute_encoder.set_buffer(0, Some(&output_buffer), 0);

    let grid_size = MTLSize { width, height, depth: 1 };

    let threadgroup_width = pipeline_state.max_total_threads_per_threadgroup();
    let threadgroup_size = MTLSize {
        width: threadgroup_width,
        height: 1,
        depth: 1,
    };

    compute_encoder.dispatch_threads(grid_size, threadgroup_size);
    compute_encoder.end_encoding();

    // --- 6. Execute and Wait for Completion ---
    command_buffer.commit();
    println!("Submitted commands to GPU. Waiting for completion...");
    command_buffer.wait_until_completed();
    println!("GPU rendering finished.");

    // --- 7. Copy Data Back to CPU and Save Image ---
    let mut image = RgbaImage::new(width as u32, height as u32);
    let ptr = output_buffer.contents() as *const f32;

    let slice_size = (width * height * 4) as usize;
    let slice = unsafe { std::slice::from_raw_parts(ptr, slice_size) };

    for y in 0..height {
        for x in 0..width {
            let index = ((y * width + x) * 4) as usize;
            let r = (slice[index] * 255.0) as u8;
            let g = (slice[index + 1] * 255.0) as u8;
            let b = (slice[index + 2] * 255.0) as u8;
            image.put_pixel(x as u32, y as u32, Rgba([r, g, b, 255]));
        }
    }

    image.save("metal_gradient.png").expect("Failed to save image.");
    println!("Image saved to metal_gradient.png");
}
