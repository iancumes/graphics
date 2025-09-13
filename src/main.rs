
use nalgebra_glm::{Vec3, normalize};
use minifb::{Key, Window, WindowOptions};
use std::time::Duration;
use std::f32::consts::PI;

mod framebuffer;
mod ray_intersect;
mod color;
mod camera;
mod material;
mod cube;
mod texture;

use framebuffer::Framebuffer;
use ray_intersect::{Intersect, RayIntersect};
use color::Color;
use camera::Camera;
use material::Material;
use cube::Cube;
use texture::Texture;

fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Box<dyn RayIntersect>],
) -> Color {
    let mut closest = Intersect::empty();
    let mut zbuffer = f32::INFINITY;

    for obj in objects {
        let i = obj.ray_intersect(ray_origin, ray_direction);
        if i.is_intersecting && i.distance < zbuffer {
            zbuffer = i.distance;
            closest = i;
        }
    }

    if !closest.is_intersecting {
        return Color::new(20, 30, 40);
    }

    closest.material.color_at(closest.u, closest.v)
}

fn render(framebuffer: &mut Framebuffer, objects: &[Box<dyn RayIntersect>], camera: &Camera) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI / 3.0;
    let perspective_scale = (fov * 0.5).tan();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            let ray_dir = normalize(&Vec3::new(screen_x, screen_y, -1.0));
            let ray_dir = camera.basis_change(&ray_dir);

            let pixel_color = cast_ray(&camera.eye, &ray_dir, objects);
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
        "Rust Graphics - Textured Cube (Free-Fly)",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();

    window.set_position(500, 300);
    window.update();

    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 3.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0)
    );

    let material = match Texture::from_file("assets/texture2.png") {
        Ok(tex) => Material::from_texture(tex),
        Err(_)  => Material::from_color(Color::new(200, 200, 220)),
    };

    let size = 2.0;
    let half = size * 0.5;
    let cube = Cube::new(
        Vec3::new(-half, -half, -4.0 - half),
        Vec3::new( half,  half, -4.0 + half),
        material
    );

    let objects: Vec<Box<dyn RayIntersect>> = vec![ Box::new(cube) ];

    let mut move_speed = 0.08f32;
    let yaw_speed = 0.03f32;
    let pitch_speed = 0.03f32;

    while window.is_open() {
        if window.is_key_down(Key::Escape) { break; }

        if window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift) {
            move_speed = 0.16;
        } else {
            move_speed = 0.08;
        }

        if window.is_key_down(Key::Left)  { camera.yaw_pitch( yaw_speed, 0.0); }
        if window.is_key_down(Key::Right) { camera.yaw_pitch(-yaw_speed, 0.0); }
        if window.is_key_down(Key::Up)    { camera.yaw_pitch(0.0,  pitch_speed); }
        if window.is_key_down(Key::Down)  { camera.yaw_pitch(0.0, -pitch_speed); }

        if window.is_key_down(Key::W) { camera.move_local( move_speed, 0.0, 0.0); }
        if window.is_key_down(Key::S) { camera.move_local(-move_speed, 0.0, 0.0); }
        if window.is_key_down(Key::D) { camera.move_local(0.0,  move_speed, 0.0); }
        if window.is_key_down(Key::A) { camera.move_local(0.0, -move_speed, 0.0); }
        if window.is_key_down(Key::R) { camera.move_local(0.0, 0.0,  move_speed); }
        if window.is_key_down(Key::F) { camera.move_local(0.0, 0.0, -move_speed); }

        render(&mut framebuffer, &objects, &camera);
        window.update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height).unwrap();
        std::thread::sleep(frame_delay);
    }
}
