
use nalgebra_glm::Vec3;
use crate::ray_intersect::{RayIntersect, Intersect};
use crate::material::Material;

pub struct Cube {
    pub min: Vec3,
    pub max: Vec3,
    pub material: Material,
}

impl Cube {
    pub fn new(min: Vec3, max: Vec3, material: Material) -> Self {
        Cube { min, max, material }
    }
}

fn near_equal(a: f32, b: f32, eps: f32) -> bool {
    (a - b).abs() < eps
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ro: &Vec3, rd: &Vec3) -> Intersect {
        let inv = Vec3::new(1.0/rd.x, 1.0/rd.y, 1.0/rd.z);

        let t1 = (self.min.x - ro.x) * inv.x;
        let t2 = (self.max.x - ro.x) * inv.x;
        let t3 = (self.min.y - ro.y) * inv.y;
        let t4 = (self.max.y - ro.y) * inv.y;
        let t5 = (self.min.z - ro.z) * inv.z;
        let t6 = (self.max.z - ro.z) * inv.z;

        let tmin_x = t1.min(t2);
        let tmax_x = t1.max(t2);
        let tmin_y = t3.min(t4);
        let tmax_y = t3.max(t4);
        let tmin_z = t5.min(t6);
        let tmax_z = t5.max(t6);

        let tmin = tmin_x.max(tmin_y).max(tmin_z);
        let tmax = tmax_x.min(tmax_y).min(tmax_z);

        if tmax < 0.0 || tmin > tmax {
            return Intersect::empty();
        }

        let t = if tmin >= 0.0 { tmin } else { tmax };
        if t < 0.0 { return Intersect::empty(); }

        let p = ro + rd * t;

        // Determine normal by checking which face we hit (epsilon comparison)
        let eps = 1e-4;
        let mut normal = Vec3::new(0.0, 0.0, 0.0);

        let mut u = 0.0;
        let mut v = 0.0;

        if near_equal(p.x, self.min.x, eps) {
            normal = Vec3::new(-1.0, 0.0, 0.0);
            // map (z,y) to (u,v)
            u = (p.z - self.min.z) / (self.max.z - self.min.z);
            v = (p.y - self.min.y) / (self.max.y - self.min.y);
        } else if near_equal(p.x, self.max.x, eps) {
            normal = Vec3::new( 1.0, 0.0, 0.0);
            u = 1.0 - (p.z - self.min.z) / (self.max.z - self.min.z);
            v = (p.y - self.min.y) / (self.max.y - self.min.y);
        } else if near_equal(p.y, self.min.y, eps) {
            normal = Vec3::new(0.0, -1.0, 0.0);
            // map (x,z)
            u = (p.x - self.min.x) / (self.max.x - self.min.x);
            v = (p.z - self.min.z) / (self.max.z - self.min.z);
        } else if near_equal(p.y, self.max.y, eps) {
            normal = Vec3::new(0.0,  1.0, 0.0);
            u = (p.x - self.min.x) / (self.max.x - self.min.x);
            v = 1.0 - (p.z - self.min.z) / (self.max.z - self.min.z);
        } else if near_equal(p.z, self.min.z, eps) {
            normal = Vec3::new(0.0, 0.0, -1.0);
            // map (x,y)
            u = (p.x - self.min.x) / (self.max.x - self.min.x);
            v = (p.y - self.min.y) / (self.max.y - self.min.y);
        } else if near_equal(p.z, self.max.z, eps) {
            normal = Vec3::new(0.0, 0.0,  1.0);
            u = 1.0 - (p.x - self.min.x) / (self.max.x - self.min.x);
            v = (p.y - self.min.y) / (self.max.y - self.min.y);
        }

        Intersect::with_uv(p, normal, t, self.material.clone(), u, v)
    }
}
