use nalgebra_glm::{Vec3, normalize, cross, rotate_vec3};

pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
}

impl Camera {
    pub fn new(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        let mut cam = Camera { eye, target, up: normalize(&up) };
        cam.reorthonormalize();
        cam
    }

    #[inline]
    fn forward(&self) -> Vec3 {
        normalize(&(self.target - self.eye))
    }

    #[inline]
    fn right(&self) -> Vec3 {
        normalize(&cross(&self.forward(), &self.up))
    }

    fn reorthonormalize(&mut self) {
        let f = self.forward();
        let r = normalize(&cross(&f, &self.up));
        let u = normalize(&cross(&r, &f));
        self.up = u;
    }

    /// Transforma una dirección de espacio cámara a espacio mundo.
    /// La cámara mira hacia -Z en espacio cámara.
    pub fn basis_change(&self, local: &Vec3) -> Vec3 {
        let f = self.forward();
        let r = self.right();
        let u = self.up;
        // world_dir = r*lx + u*ly + (-f)*lz
        let world = r * local.x + u * local.y + (-f) * local.z;
        normalize(&world)
    }

    /// Orbitar alrededor del 'target' (queda por si lo necesitás).
    pub fn orbit(&mut self, dtheta: f32, dphi: f32) {
        let center = self.target;
        let to_eye = self.eye - center;

        // yaw alrededor de 'up' (3D correcto)
        let yawed = rotate_vec3(&to_eye, dtheta, &self.up);

        // pitch alrededor del eje 'right' tras yaw
        let right = normalize(&cross(&normalize(&(-yawed)), &self.up));
        let pitched = rotate_vec3(&yawed, dphi, &right);

        self.eye = center + pitched;
        self.reorthonormalize();
    }

    /// Girar la dirección de vista en su lugar (first-person look): ojo fijo, mueve target.
    pub fn yaw_pitch(&mut self, dyaw: f32, dpitch: f32) {
        let dist = nalgebra_glm::distance(&self.eye, &self.target).max(1e-5);
        let f0 = self.forward();

        // yaw alrededor del 'up' actual
        let f1 = normalize(&rotate_vec3(&f0, dyaw, &self.up));

        // pitch alrededor de 'right'
        let right = normalize(&cross(&f1, &self.up));
        let mut f2 = normalize(&rotate_vec3(&f1, dpitch, &right));

        // Evitar volteo cerca de los polos
        let dot_up = nalgebra_glm::dot(&f2, &self.up);
        let limit = 0.995; // ~84°
        if dot_up.abs() > limit {
            f2 = if dot_up > 0.0 { self.up * limit } else { -self.up * limit };
            // pequeño empuje hacia el forward previo para evitar lock
            f2 = normalize(&(f2 + f1 * 0.01));
        }

        self.target = self.eye + f2 * dist;
        self.reorthonormalize();
    }

    /// Mover la cámara en su base local: (fwd, right, up) en unidades del mundo.
    pub fn move_local(&mut self, fwd: f32, right_amt: f32, up_amt: f32) {
        let f = self.forward();
        let r = self.right();
        let u = self.up;
        let delta = f * fwd + r * right_amt + u * up_amt;
        self.eye += delta;
        self.target += delta;
    }
}
