use nalgebra_glm::{Vec3, normalize};
use minifb::{Key, Window, WindowOptions};
use std::time::Duration;
use std::f32::consts::PI;

mod framebuffer;
mod ray_intersect;
mod sphere; 
mod color;
mod camera;
mod light;
mod material;

use framebuffer::Framebuffer;
use sphere::Sphere;
use color::Color;
use ray_intersect::{Intersect, RayIntersect};
use camera::Camera;
use light::Light;
use material::Material;

const SHADOW_BIAS: f32 = 1e-4;
const MAX_DEPTH: u32 = 4;


fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - 2.0 * incident.dot(normal) * normal
}

fn refract(incident: &Vec3, normal: &Vec3, ior: f32) -> Option<Vec3> {
    // Snell’s law with inside/outside handling
    let mut n = *normal;
    let mut cosi = incident.dot(&n).clamp(-1.0, 1.0);
    let (mut etai, mut etat) = (1.0_f32, ior);
    if cosi > 0.0 {
        // we are inside the medium
        std::mem::swap(&mut etai, &mut etat);
        n = -n;
    } else {
        cosi = -cosi;
    }
    let eta = etai / etat;
    let k = 1.0 - eta * eta * (1.0 - cosi * cosi);
    if k < 0.0 {
        None // total internal reflection
    } else {
        Some(eta * *incident + (eta * cosi - k.sqrt()) * n)
    }
}

fn fresnel(incident: &Vec3, normal: &Vec3, ior: f32) -> f32 {
    // Schlick approximation
    let mut cosi = incident.dot(normal).clamp(-1.0, 1.0);
    let (mut etai, mut etat) = (1.0_f32, ior);
    if cosi > 0.0 {
        std::mem::swap(&mut etai, &mut etat);
    }
    // sine of transmission angle using Snell’s law
    let sint = etai / etat * (1.0 - cosi * cosi).max(0.0).sqrt();
    if sint >= 1.0 {
        1.0 // total internal reflection
    } else {
        cosi = cosi.abs();
        let r0 = ((etat - etai) / (etat + etai)).powi(2);
        r0 + (1.0 - r0) * (1.0 - cosi).powi(5)
    }
}

fn cast_shadow(
    intersect: &Intersect,
    light: &Light,
    objects: &[Sphere],
) -> f32 {
    let light_dir = (light.position - intersect.point).normalize();
    let light_distance = (light.position - intersect.point).magnitude();

    let offset_normal = intersect.normal * SHADOW_BIAS;
    let shadow_ray_origin = if light_dir.dot(&intersect.normal) < 0.0 {
        intersect.point - offset_normal
    } else {
        intersect.point + offset_normal
    };

    let mut shadow_intensity = 0.0;

    for object in objects {
        let shadow_intersect = object.ray_intersect(&shadow_ray_origin, &light_dir);
        if shadow_intersect.is_intersecting && shadow_intersect.distance < light_distance {
            let distance_ratio = shadow_intersect.distance / light_distance;
            shadow_intensity = 1.0 - distance_ratio.powf(2.0).min(1.0);
            break;
        }
    }

    shadow_intensity
}



pub fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Sphere],
    light: &Light,
    depth: u32,
) -> Color {
    if depth > MAX_DEPTH {
        return Color::new(0, 10, 100); // background
    }
    let mut intersect = Intersect::empty();
    let mut zbuffer = f32::INFINITY;

    for object in objects {
        let i = object.ray_intersect(ray_origin, ray_direction);
        if i.is_intersecting && i.distance < zbuffer {
            zbuffer = i.distance;
            intersect = i;
        }
    }

    if !intersect.is_intersecting {
        // return default sky box color
        return Color::new(200, 12, 36);
    }

    let light_dir = (light.position - intersect.point).normalize();
    let view_dir = (ray_origin - intersect.point).normalize();
    let reflect_dir_l = reflect(&-light_dir, &intersect.normal);

    let shadow_intensity = cast_shadow(&intersect, light, objects);
    let light_intensity = light.intensity * (1.0 - shadow_intensity);

    let diffuse_intensity = intersect.normal.dot(&light_dir).max(0.0).min(1.0);
    let diffuse = intersect.material.diffuse
        * intersect.material.albedo[0]
        * diffuse_intensity
        * light_intensity;

    let specular_intensity = view_dir.dot(&reflect_dir_l).max(0.0).powf(intersect.material.specular);
    let specular = light.color
        * intersect.material.albedo[1]
        * specular_intensity
        * light_intensity;

    let local_color = diffuse + specular;

    // reflection / refraction
    let mut result = local_color;

    // Early out if no mirror or glass
    let has_reflect = intersect.material.albedo[2] > 0.0;
    let has_refract  = intersect.material.albedo[3] > 0.0;

    if has_reflect || has_refract {
        let normal = intersect.normal;
        let reflect_dir = reflect(ray_direction, &normal).normalize();

        // Offset origins to avoid self-intersections
        let reflect_origin = if reflect_dir.dot(&normal) < 0.0 {
            intersect.point - normal * SHADOW_BIAS
        } else {
            intersect.point + normal * SHADOW_BIAS
        };

        // Fresnel factor to balance reflect vs refract
        let kr = fresnel(ray_direction, &normal, intersect.material.ior);
        let mut reflect_color = Color::new(0, 0, 0);
        let mut refract_color = Color::new(0, 0, 0);

        if has_reflect {
            reflect_color = cast_ray(&reflect_origin, &reflect_dir, objects, light, depth + 1);
        }

        if has_refract {
            if let Some(refract_dir) = refract(ray_direction, &normal, intersect.material.ior) {
                let refract_dir_n = refract_dir.normalize();
                let refract_origin = if refract_dir_n.dot(&normal) < 0.0 {
                    intersect.point - normal * SHADOW_BIAS
                } else {
                    intersect.point + normal * SHADOW_BIAS
                };
                refract_color = cast_ray(&refract_origin, &refract_dir_n, objects, light, depth + 1);
            }
        }

        // weights: use albedo[2] for reflection, albedo[3] for refraction,
        // modulated by Fresnel (kr).
        let reflection_weight = intersect.material.albedo[2] * kr;
        let refraction_weight = intersect.material.albedo[3] * (1.0 - kr);

        result = result
            + reflect_color * reflection_weight
            + refract_color * refraction_weight;
    }

    result
}

pub fn render(framebuffer: &mut Framebuffer, objects: &[Sphere], camera: &Camera, light: &Light) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI/3.0;
    let perspective_scale = (fov * 0.5).tan();

    // random number generator
    // let mut rng = rand::thread_rng();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            // if rng.gen_range(0.0..1.0) < 0.3 {
            //     // we skip 30% of the points
            //     continue;
            // }

            // Map the pixel coordinate to screen space [-1, 1]
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            // Adjust for aspect ratio and perspective 
            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            // Calculate the direction of the ray for this pixel
            let ray_direction = normalize(&Vec3::new(screen_x, screen_y, -1.0));

            // Apply camera rotation to the ray direction
            let rotated_direction = camera.basis_change(&ray_direction);

            // Cast the ray and get the pixel color
            let pixel_color = cast_ray(&camera.eye, &rotated_direction, objects, light, 0);

            // Draw the pixel on screen with the returned color
            framebuffer.set_current_color(pixel_color.to_hex());
            framebuffer.point(x, y);
        }
    }
}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Rust Graphics - Raytracer Example",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();

    // move the window around
    window.set_position(500, 500);
    window.update();

    let rubber = Material::new(
        Color::new(255, 30, 30),
        1.0,
        [0.8, 0.5, 0.1, 0.0], // diffuse,specular,reflect, refract
        1.0                    // ior (no refraction)
    );


    let ivory = Material::new(
        Color::new(160, 50, 50),
        50.0,
        [0.6, 0.1, 0.3, 0.0],
        1.0
    );

    let glass = Material::new(
        Color::new(70, 150, 150),
        125.0,
        [0.0, 0.1, 0.05, 0.9], // mostly refractive, tiny spec + reflect
        1.5                    // glass-ish IOR
    );

    let objects = [
        Sphere { center: Vec3::new(0.0, 0.0, 0.0),   radius: 1.0, material: rubber },
        Sphere { center: Vec3::new(0.0, 0.0, 1.8),  radius: 0.5, material: glass  },
        Sphere { center: Vec3::new(1.0, 1.0, 3.0), radius: 0.7, material: rubber },
        Sphere { center: Vec3::new(-2.0, 2.0, -5.0), radius: 1.0, material: ivory },
    ];

    // Initialize camera
    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 5.0),  // eye: Initial camera position
        Vec3::new(0.0, 0.0, 0.0),  // center: Point the camera is looking at (origin)
        Vec3::new(0.0, 1.0, 0.0)   // up: World up vector
    );
    let rotation_speed = PI/50.0;

    let light = Light::new(
        Vec3::new(5.0, 8.0, -5.0),
        Color::new(100, 100, 100),
        8.0
    );

    while window.is_open() {
        // listen to inputs
        if window.is_key_down(Key::Escape) {
            break;
        }

        //  camera orbit controls
        if window.is_key_down(Key::Left) {
            camera.orbit(rotation_speed, 0.0);
        }
        if window.is_key_down(Key::Right) {
            camera.orbit(-rotation_speed, 0.0);
        }
        if window.is_key_down(Key::Up) {
            camera.orbit(0.0, -rotation_speed);
        }
        if window.is_key_down(Key::Down) {
            camera.orbit(0.0, rotation_speed);
        }

        // draw some points
        render(&mut framebuffer, &objects, &camera, &light);


        // update the window with the framebuffer contents
        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}
