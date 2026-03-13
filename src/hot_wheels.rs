use nalgebra::Vector3;
use image::{ImageBuffer, RgbImage};
use std::f64::consts::PI;
use rayon::prelude::*;
use indicatif::{ProgressBar, ProgressStyle, ParallelProgressIterator};

type Vec3 = Vector3<f64>;

// --- Helper Structs and Trait Implementations (Unchanged) ---
struct Ray { origin: Vec3, direction: Vec3, }
#[derive(Clone, Copy)] struct Light { position: Vec3, color: Vec3, }
#[derive(Clone, Copy)] struct Material { color: Vec3, albedo: f64, specular: Vec3, shininess: f64, reflectivity: f64, is_wood_table: bool, }
trait Hittable: Send + Sync { fn hit(&self, ray: &Ray) -> Option<(f64, Vec3, Vec3, Material)>; }
struct Scene { objects: Vec<Box<dyn Hittable>>, lights: Vec<Light>, }
struct Plane { point: Vec3, normal: Vec3, material: Material, }
impl Hittable for Plane { fn hit(&self, ray: &Ray) -> Option<(f64, Vec3, Vec3, Material)> { let denominator = self.normal.dot(&ray.direction); if denominator.abs() > 0.001 { let t = (self.point - ray.origin).dot(&self.normal) / denominator; if t > 0.001 { let point = ray.origin + t * ray.direction; return Some((t, point, self.normal, self.material)); } } None } }
struct Quad { corner: Vec3, u: Vec3, v: Vec3, normal: Vec3, material: Material, }
impl Quad { fn new(corner: Vec3, u: Vec3, v: Vec3, material: Material) -> Self { Self { corner, u, v, normal: u.cross(&v).normalize(), material, } } }
impl Hittable for Quad { fn hit(&self, ray: &Ray) -> Option<(f64, Vec3, Vec3, Material)> { let plane = Plane { point: self.corner, normal: self.normal, material: self.material }; if let Some((dist, point, _, _)) = plane.hit(ray) { let hit_vec = point - self.corner; let alpha = hit_vec.dot(&self.u) / self.u.dot(&self.u); let beta = hit_vec.dot(&self.v) / self.v.dot(&self.v); if (0.0..=1.0).contains(&alpha) && (0.0..=1.0).contains(&beta) { return Some((dist, point, self.normal, self.material)); } } None } }

// --- Wood Texture and Trace Ray functions are UNCHANGED ---
fn get_wood_color(point: &Vec3) -> Vec3 { let light_wood = Vec3::new(0.85, 0.7, 0.45); let dark_wood = Vec3::new(0.6, 0.45, 0.25); let noise_val = (point.z * 2.0 + (point.x * 10.0).sin() * 2.0).sin() + (point.z * 5.0 + (point.x * 25.0).sin() * 1.5).sin() * 0.3; let grain_factor = (noise_val + 1.3) / 2.6; light_wood.lerp(&dark_wood, grain_factor) }
fn reflect(v: &Vec3, n: &Vec3) -> Vec3 { v - 2.0 * v.dot(n) * n }
fn trace_ray(ray: &Ray, scene: &Scene, depth: i32) -> Vec3 { if depth <= 0 { return Vec3::zeros(); } let mut closest_hit: Option<(f64, Vec3, Vec3, Material)> = None; for object in &scene.objects { if let Some(hit) = object.hit(ray) { if closest_hit.is_none() || hit.0 < closest_hit.unwrap().0 { closest_hit = Some(hit); } } } if let Some((_dist, point, normal, material)) = closest_hit { let mut final_color = Vec3::zeros(); let view_dir = -ray.direction; let surface_color = if material.is_wood_table { get_wood_color(&point) } else { material.color }; for light in &scene.lights { let light_dir = (light.position - point).normalize(); let n_dot_l = normal.dot(&light_dir).max(0.0); let diffuse = surface_color.component_mul(&light.color) * n_dot_l * material.albedo; let reflected_light_dir = reflect(&-light_dir, &normal); let r_dot_v = reflected_light_dir.dot(&view_dir).max(0.0); let specular = material.specular.component_mul(&light.color) * r_dot_v.powf(material.shininess); final_color += diffuse + specular; } let reflected_ray_dir = reflect(&ray.direction, &normal); let reflected_color = if material.reflectivity > 0.0 { let reflect_ray = Ray { origin: point + normal * 0.001, direction: reflected_ray_dir }; trace_ray(&reflect_ray, scene, depth - 1) * material.reflectivity } else { Vec3::zeros() }; return final_color + reflected_color; } else { Vec3::new(0.1, 0.1, 0.1) } }

fn main() {
    // --- Image Setup ---
    let aspect_ratio = 16.0 / 9.0;
    let width = 1280;
    let height = (width as f64 / aspect_ratio) as u32;

    // --- Materials & Scene Geometry (Built once for efficiency) ---
    let orange_plastic = Material { color: Vec3::new(1.0, 0.35, 0.0), albedo: 0.8, specular: Vec3::new(1.0, 1.0, 1.0), shininess: 30.0, reflectivity: 0.05, is_wood_table: false };
    let wood_table_material = Material { color: Vec3::new(0.0, 0.0, 0.0), albedo: 0.9, specular: Vec3::new(0.2, 0.2, 0.2), shininess: 10.0, reflectivity: 0.0, is_wood_table: true };
    let table = Box::new(Plane { point: Vec3::new(0.0, 0.0, 0.0), normal: Vec3::new(0.0, 1.0, 0.0), material: wood_table_material });
    let mut track_pieces: Vec<Box<dyn Hittable>> = vec![];
    let track_width = 0.5; let lip_height = 0.2; let lip_thickness = 0.03; let track_y = 0.5; let straight_len = 6.0; let curve_radius = 2.5; let num_segments = 150;
    let mut path_points: Vec<Vec3> = vec![];
    let curve1_center = Vec3::new(straight_len / 2.0, track_y, 0.0); let curve2_center = Vec3::new(-straight_len / 2.0, track_y, 0.0);
    for i in 0..=num_segments { path_points.push(Vec3::new(-straight_len / 2.0 + (i as f64 / num_segments as f64) * straight_len, track_y, -curve_radius)); }
    for i in 1..=num_segments { let p = i as f64 / num_segments as f64; let angle = -PI / 2.0 + p * PI; path_points.push(curve1_center + Vec3::new(angle.cos() * curve_radius, 0.0, angle.sin() * curve_radius)); }
    for i in 1..=num_segments { path_points.push(Vec3::new(straight_len / 2.0 - (i as f64 / num_segments as f64) * straight_len, track_y, curve_radius)); }
    for i in 1..=num_segments { let p = i as f64 / num_segments as f64; let angle = PI / 2.0 + p * PI; path_points.push(curve2_center + Vec3::new(angle.cos() * curve_radius, 0.0, angle.sin() * curve_radius)); }
    for i in 0..(path_points.len() - 1) {
        let p1 = path_points[i]; let p2 = path_points[i+1]; let v = p2 - p1;
        if v.magnitude() < 1e-6 { continue; }
        let u_dir = v.cross(&Vec3::new(0.0, 1.0, 0.0)).normalize(); let u = u_dir * track_width; let corner = p1 - u / 2.0;
        track_pieces.push(Box::new(Quad::new(corner, u, v, orange_plastic)));
        let lip_v = Vec3::new(0.0, lip_height, 0.0); let thickness_vec = u_dir * lip_thickness;
        let left_lip_inner = corner; let left_lip_outer = corner - thickness_vec;
        track_pieces.push(Box::new(Quad::new(left_lip_inner, v, lip_v, orange_plastic))); track_pieces.push(Box::new(Quad::new(left_lip_outer, v, lip_v, orange_plastic))); track_pieces.push(Box::new(Quad::new(left_lip_outer + lip_v, v, thickness_vec, orange_plastic)));
        let right_lip_inner = corner + u; let right_lip_outer = corner + u + thickness_vec;
        track_pieces.push(Box::new(Quad::new(right_lip_inner, v, lip_v, orange_plastic))); track_pieces.push(Box::new(Quad::new(right_lip_outer, v, lip_v, orange_plastic))); track_pieces.push(Box::new(Quad::new(right_lip_inner + lip_v, v, thickness_vec, orange_plastic)));
    }
    let mut all_objects: Vec<Box<dyn Hittable>> = vec![table]; all_objects.extend(track_pieces);
    let final_scene = Scene { objects: all_objects, lights: vec! [ Light { position: Vec3::new(0.0, 10.0, 0.0), color: Vec3::new(0.8, 0.8, 0.8) }, Light { position: Vec3::new(10.0, 5.0, 10.0), color: Vec3::new(0.4, 0.4, 0.5) }, ], };

    // --- Animation Setup ---
    let num_frames = 120;

    let start_lookfrom = Vec3::new(7.0, 4.0, 0.0);
    let end_lookfrom = Vec3::new(0.0, 0.9, -2.5);

    let start_lookat = Vec3::new(0.0, track_y, -curve_radius);
    // --- CHANGE IS HERE ---
    // The final target is now the corner of the FAR curve (on the left side).
    let end_lookat = Vec3::new(-straight_len / 2.0, track_y, -curve_radius);

    // --- Main Animation & Render Loop ---
    println!("Rendering {} frames with parallel scanlines...", num_frames);
    for frame in 0..num_frames {
        println!("\n-- Starting frame {} of {} --", frame + 1, num_frames);

        let progress = frame as f64 / (num_frames - 1) as f64;

        let lookfrom = start_lookfrom.lerp(&end_lookfrom, progress);
        let lookat = start_lookat.lerp(&end_lookat, progress);

        let vup = Vec3::new(0.0, 1.0, 0.0);
        let vfov_degrees = 60.0;

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
                    let pixel_color = trace_ray(&ray, &final_scene, 5);
                    [(pixel_color.x.clamp(0.0, 1.0) * 255.0) as u8, (pixel_color.y.clamp(0.0, 1.0) * 255.0) as u8, (pixel_color.z.clamp(0.0, 1.0) * 255.0) as u8]
                }).collect::<Vec<u8>>()
            }).collect();

        let img: RgbImage = ImageBuffer::from_raw(width, height, pixels).expect("Could not create image from pixel buffer");
        let filename = format!("output/anim_parallel_{:03}.png", frame);
        println!("Frame render complete. Saving to {}...", filename);
        img.save(&filename).expect("Failed to save image.");
    }

    println!("\nAll animation frames rendered successfully.");
}
