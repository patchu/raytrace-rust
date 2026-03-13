use nalgebra::Vector3;
use image::{ImageBuffer, RgbImage};
use std::f64::consts::PI;
use rayon::prelude::*;
use indicatif::{ProgressBar, ProgressStyle, ParallelProgressIterator};

type Vec3 = Vector3<f64>;

// --- NO CHANGES TO Ray, Material, Hittable, Sphere, Plane, Light, Scene ---
struct Ray {
    origin: Vec3,
    direction: Vec3,
}

#[derive(Clone, Copy)]
struct Material {
    color: Vec3,
    albedo: f64,
    specular: Vec3,
    shininess: f64,
    reflectivity: f64,
}

trait Hittable: Send + Sync {
    fn hit(&self, ray: &Ray) -> Option<(f64, Vec3, Vec3, Material)>;
}

struct Sphere {
    center: Vec3,
    radius: f64,
    material: Material,
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray) -> Option<(f64, Vec3, Vec3, Material)> {
        let oc = ray.origin - self.center;
        let a = ray.direction.dot(&ray.direction);
        let b = 2.0 * oc.dot(&ray.direction);
        let c = oc.dot(&oc) - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            None
        } else {
            let t = (-b - discriminant.sqrt()) / (2.0 * a);
            if t > 0.001 {
                let point = ray.origin + t * ray.direction;
                let normal = (point - self.center).normalize();
                Some((t, point, normal, self.material))
            } else {
                let t2 = (-b + discriminant.sqrt()) / (2.0 * a);
                if t2 > 0.001 {
                    let point = ray.origin + t2 * ray.direction;
                    let normal = (point - self.center).normalize();
                    Some((t2, point, normal, self.material))
                } else {
                    None
                }
            }
        }
    }
}

struct Plane {
    point: Vec3,
    normal: Vec3,
    material: Material,
}

impl Hittable for Plane {
    fn hit(&self, ray: &Ray) -> Option<(f64, Vec3, Vec3, Material)> {
        let denominator = self.normal.dot(&ray.direction);
        if denominator.abs() > 0.001 {
            let t = (self.point - ray.origin).dot(&self.normal) / denominator;
            if t > 0.001 {
                let point = ray.origin + t * ray.direction;
                return Some((t, point, self.normal, self.material));
            }
        }
        None
    }
}

#[derive(Clone)]
struct Light {
    position: Vec3,
    color: Vec3,
}

struct Scene {
    objects: Vec<Box<dyn Hittable>>,
    light: Light,
}


/// A simple graphics function to create a smooth transition between two values.
fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Calculates the background color, including the sunset gradient and clouds.
fn get_background_color(ray_direction: &Vec3) -> Vec3 {
    // --- 1. Create the sunset gradient ---
    let zenith_color = Vec3::new(0.3, 0.4, 0.7);
    let horizon_color = Vec3::new(1.0, 0.7, 0.5);
    // Blend based on the ray's y-direction.
    let t = 0.5 * (ray_direction.y + 1.0);
    let sky_color = zenith_color.lerp(&horizon_color, t * t); // t*t makes horizon band tighter

    // --- 2. Add procedural clouds ---
    let cloud_color = Vec3::new(1.0, 1.0, 1.0);
    // Use sine functions to create a simple, layered noise pattern.
    // Multiplying by different frequencies creates different levels of detail.
    let noise = (ray_direction.x * 8.0).sin() * (ray_direction.z * 8.0).sin();
    let large_blobs = (ray_direction.x * 3.0).sin() * (ray_direction.z * 3.0).sin();
    let combined_noise = (noise + large_blobs * 0.5).abs();

    // Use smoothstep to create soft-edged clouds from the noise.
    // Values from 0.5 to 0.7 in the noise will become clouds.
    let cloud_alpha = smoothstep(0.5, 0.7, combined_noise);

    // Blend the sky color with the cloud color based on the noise.
    sky_color.lerp(&cloud_color, cloud_alpha)
}


// --- reflect function is UNCHANGED ---
fn reflect(v: &Vec3, n: &Vec3) -> Vec3 {
    v - 2.0 * v.dot(n) * n
}


fn trace_ray(ray: &Ray, scene: &Scene, depth: i32) -> Vec3 {
    if depth <= 0 { return Vec3::zeros(); }

    let mut closest_hit: Option<(f64, Vec3, Vec3, Material)> = None;

    for object in &scene.objects {
        if let Some((dist, point, normal, material)) = object.hit(ray) {
            if closest_hit.is_none() || dist < closest_hit.unwrap().0 {
                closest_hit = Some((dist, point, normal, material));
            }
        }
    }

    if let Some((_dist, point, normal, material)) = closest_hit {
        let current_material = if normal.y == 1.0 {
            let is_white_square = (point.x.floor() + point.z.floor()) as i32 % 2 == 0;
            if is_white_square { material } else { Material { color: Vec3::new(0.1, 0.1, 0.1), albedo: 0.8, ..material } }
        } else {
            material
        };

        let light_dir = (scene.light.position - point).normalize();
        let view_dir = -ray.direction;
        let n_dot_l = normal.dot(&light_dir).max(0.0);
        let diffuse = current_material.color.component_mul(&scene.light.color) * n_dot_l * current_material.albedo;
        let reflected_light_dir = reflect(&-light_dir, &normal);
        let r_dot_v = reflected_light_dir.dot(&view_dir).max(0.0);
        let specular = current_material.specular.component_mul(&scene.light.color) * r_dot_v.powf(current_material.shininess);
        let reflected_ray_dir = reflect(&ray.direction, &normal);
        let reflected_color = if current_material.reflectivity > 0.0 {
            let reflect_ray = Ray { origin: point + normal * 0.001, direction: reflected_ray_dir };
            trace_ray(&reflect_ray, scene, depth - 1) * current_material.reflectivity
        } else {
            Vec3::zeros()
        };

        return diffuse + specular + reflected_color;

    } else {
        // --- THIS IS THE ONLY CHANGE IN THIS FUNCTION ---
        // Instead of a solid color, call our new background function.
        return get_background_color(&ray.direction);
    }
}


// This is the only function you need to replace.

fn main() {
    // --- Image Setup ---
    let aspect_ratio = 16.0 / 9.0;
    let width = 1920;
    let height = (width as f64 / aspect_ratio) as u32;

    // --- Materials (Unchanged) ---
    let red_shiny = Material { color: Vec3::new(1.0, 0.0, 0.0), albedo: 0.5, specular: Vec3::new(0.7, 0.7, 0.7), shininess: 50.0, reflectivity: 0.3 };
    let blue_shiny = Material { color: Vec3::new(0.2, 0.3, 1.0), albedo: 0.4, specular: Vec3::new(0.8, 0.8, 0.8), shininess: 80.0, reflectivity: 0.4 };
    let silver_reflective = Material { color: Vec3::new(0.8, 0.8, 0.8), albedo: 0.1, specular: Vec3::new(1.0, 1.0, 1.0), shininess: 100.0, reflectivity: 0.8 };
    let bright_checkerboard = Material { color: Vec3::new(1.0, 1.0, 1.0), albedo: 0.8, specular: Vec3::new(0.0, 0.0, 0.0), shininess: 0.0, reflectivity: 0.0 };
    let light = Light { position: Vec3::new(-10.0, 10.0, 0.0), color: Vec3::new(1.0, 1.0, 1.0) };

    // =======================================================
    // --- ANIMATION LOOP ---
    // =======================================================
    let num_frames = 600;
    for frame in 0..num_frames {
        // --- DYNAMIC MOVEMENT SETUP ---
        let progress = frame as f64 / num_frames as f64;
        let camera_angle = progress * 2.0 * (2.0 * PI);
        let cycle_angle = progress * 2.0 * PI;

        // =======================================================
        // --- DYNAMIC SPHERE POSITIONS (WITH CORRECTION) ---
        // =======================================================
        // The baseline Y is now raised to radius (1.0) + amplitude + a small gap (0.2).
        let red_y = 1.7 + 0.5 * (5.0 * cycle_angle).sin();    // Baseline: 1.0(r) + 0.5(a) + 0.2(g) = 1.7
        let blue_y = 1.9 + 0.7 * (2.0 * cycle_angle).sin();   // Baseline: 1.0(r) + 0.7(a) + 0.2(g) = 1.9
        let silver_y = 1.8 + 0.6 * (3.0 * cycle_angle).sin(); // Baseline: 1.0(r) + 0.6(a) + 0.2(g) = 1.8

        // --- REBUILD THE SCENE FOR THIS FRAME ---
        let scene = Scene {
            objects: vec! [
                Box::new(Sphere { center: Vec3::new(0.0, red_y, -5.0), radius: 1.0, material: red_shiny }),
                Box::new(Sphere { center: Vec3::new(-2.5, blue_y, -4.5), radius: 1.0, material: blue_shiny }),
                Box::new(Sphere { center: Vec3::new(2.5, silver_y, -4.0), radius: 1.0, material: silver_reflective }),
                Box::new(Plane { point: Vec3::new(0.0, 0.0, 0.0), normal: Vec3::new(0.0, 1.0, 0.0), material: bright_checkerboard }),
            ],
            light: light.clone(),
        };

        // --- DYNAMIC CAMERA SETUP ---
        let lookat = Vec3::new(0.0, 1.5, -5.0); // Focus point raised slightly to better follow the action
        let radius = 8.0;
        let lookfrom = Vec3::new(
            lookat.x + radius * camera_angle.cos(),
            3.5,
            lookat.z + radius * camera_angle.sin()
        );

        let vup = Vec3::new(0.0, 1.0, 0.0);
        let vfov_degrees = 75.0;
        let vfov_radians = vfov_degrees * PI / 180.0;
        let h = (vfov_radians / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = aspect_ratio * viewport_height;
        let w = (lookfrom - lookat).normalize();
        let u = vup.cross(&w).normalize();
        let v = w.cross(&u);
        let horizontal = viewport_width * u;
        let vertical = viewport_height * v;
        let lower_left_corner = lookfrom - horizontal / 2.0 - vertical / 2.0 - w;

        // --- RENDER LOOP (parallel scanlines) ---
        println!("\nRendering frame {} of {}...", frame + 1, num_frames);

        let bar = ProgressBar::new(height as u64);
        bar.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-"));

        let pixels: Vec<u8> = (0..height).into_par_iter()
            .progress_with(bar)
            .flat_map(|y| {
                (0..width).flat_map(|x| {
                    let s = x as f64 / (width - 1) as f64;
                    let t = (height - 1 - y) as f64 / (height - 1) as f64;
                    let ray_direction = (lower_left_corner + s * horizontal + t * vertical - lookfrom).normalize();
                    let ray = Ray { origin: lookfrom, direction: ray_direction };
                    let pixel_color = trace_ray(&ray, &scene, 5);
                    [(pixel_color.x.clamp(0.0, 1.0) * 255.0) as u8, (pixel_color.y.clamp(0.0, 1.0) * 255.0) as u8, (pixel_color.z.clamp(0.0, 1.0) * 255.0) as u8]
                }).collect::<Vec<u8>>()
            }).collect();

        let img: RgbImage = ImageBuffer::from_raw(width, height, pixels).expect("Could not create image from pixel buffer");

        // --- SAVE THE FRAME ---
        let filename = format!("output/frame_{:03}.png", frame);
        println!("Frame render complete. Saving to {}...", filename);
        img.save(&filename).expect("Failed to save image.");
    }
    // --- END OF ANIMATION LOOP ---

    println!("\nAll frames rendered successfully.");
}
