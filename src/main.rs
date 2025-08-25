// main.rs (o el archivo donde tienes este código)

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

fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - 2.0 * incident.dot(normal) * normal
}

pub fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Sphere],
    light: &Light,
) -> Color {
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
        // color del "cielo"
        return Color::new(4, 12, 36);
    }

    let light_dir = (light.position - intersect.point).normalize();
    let view_dir = (ray_origin - intersect.point).normalize();
    let reflect_dir = reflect(&-light_dir, &intersect.normal);

    // SIN SOMBRAS: no atenuamos la luz
    let light_intensity = light.intensity;

    // Difuso
    let diffuse_intensity = intersect.normal.dot(&light_dir).max(0.0).min(1.0);
    let diffuse = intersect.material.diffuse
        * intersect.material.albedo[0]
        * diffuse_intensity
        * light_intensity;

    // Especular
    let specular_intensity = view_dir.dot(&reflect_dir).max(0.0).powf(intersect.material.specular);
    let specular = light.color
        * intersect.material.albedo[1]
        * specular_intensity
        * light_intensity;

    diffuse + specular
}

pub fn render(framebuffer: &mut Framebuffer, objects: &[Sphere], camera: &Camera, light: &Light) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI/3.0;
    let perspective_scale = (fov * 0.5).tan();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            // Mapeo a espacio de pantalla [-1, 1]
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            // Ajuste de relación de aspecto y perspectiva
            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            // Dirección del rayo
            let ray_direction = normalize(&Vec3::new(screen_x, screen_y, -1.0));

            // Rotación por la cámara
            let rotated_direction = camera.basis_change(&ray_direction);

            // Color del píxel
            let pixel_color = cast_ray(&camera.eye, &rotated_direction, objects, light);

            // Dibujo
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
        "Rust Graphics - Raytracer Example (sin sombras)",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();

    // mover la ventana
    window.set_position(500, 500);
    window.update();

    let rubber = Material::new(
        Color::new(153, 40, 30),
        1.0,
        [0.9, 0.1],
    );

    let ivory = Material::new(
        Color::new(39, 103, 252),
        50.0,
        [0.6, 0.3],
    );

    let objects = [
        Sphere { center: Vec3::new(0.0, 0.0, 0.0), radius: 1.0, material: rubber },
        Sphere { center: Vec3::new(0.0, 0.0, 1.5), radius: 0.5, material: ivory },
        // Sphere { center: Vec3::new(1.0, 1.0, 3.0), radius: 0.7, material: rubber },
        // Sphere { center: Vec3::new(-2.0, 2.0, -5.0), radius: 1.0, material: ivory },
    ];

    // Cámara
    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 5.0),  // eye
        Vec3::new(0.0, 0.0, 0.0),  // center
        Vec3::new(0.0, 1.0, 0.0)   // up
    );
    let rotation_speed = PI/50.0;

    let light = Light::new(
        Vec3::new(7.0, 4.0, 9.0),
        Color::new(252, 39, 39),
        15.0
    );

    while window.is_open() {
        // inputs
        if window.is_key_down(Key::Escape) {
            break;
        }

        // controles de órbita
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

        // render
        render(&mut framebuffer, &objects, &camera, &light);

        // actualizar ventana
        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}
