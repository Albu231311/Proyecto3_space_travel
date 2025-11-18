use nalgebra_glm::{Vec3, Mat4, perspective, look_at, ortho};
use std::f32::consts::PI;

pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub aspect_ratio: f32,
    pub near: f32,
    pub far: f32,
    
    // Controles de cámara
    pub distance_from_target: f32,
    pub horizontal_angle: f32,
    pub vertical_angle: f32,
}

impl Camera {
    pub fn new(width: f32, height: f32) -> Self {
        Camera {
            position: Vec3::new(0.0, 500.0, 1000.0),
            target: Vec3::new(0.0, 0.0, 0.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            fov: 35.0_f32.to_radians(), // Reducir FOV para menos distorsión
            aspect_ratio: width / height,
            near: 0.1,
            far: 5000.0,
            distance_from_target: 1200.0, // Alejarse más para ver mejor
            horizontal_angle: 0.0,
            vertical_angle: 0.3, // Ángulo más suave
        }
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        look_at(&self.position, &self.target, &self.up)
    }

    pub fn get_projection_matrix(&self) -> Mat4 {
        // CAMBIO TEMPORAL: Usar proyección ortográfica para eliminar distorsión
        let left = -600.0;
        let right = 600.0; 
        let bottom = -400.0;
        let top = 400.0;
        
        nalgebra_glm::ortho(left, right, bottom, top, self.near, self.far)
        
        // Comentamos la proyección perspectiva por ahora:
        // perspective(self.fov, self.aspect_ratio, self.near, self.far)
    }

    pub fn get_view_projection_matrix(&self) -> Mat4 {
        self.get_projection_matrix() * self.get_view_matrix()
    }

    pub fn update_position(&mut self) {
        // Calcular posición de la cámara basada en ángulos esféricos
        let x = self.distance_from_target * self.vertical_angle.cos() * self.horizontal_angle.cos();
        let y = self.distance_from_target * self.vertical_angle.sin();
        let z = self.distance_from_target * self.vertical_angle.cos() * self.horizontal_angle.sin();
        
        self.position = self.target + Vec3::new(x, y, z);
    }

    pub fn rotate(&mut self, delta_horizontal: f32, delta_vertical: f32) {
        self.horizontal_angle += delta_horizontal;
        self.vertical_angle += delta_vertical;
        
        // Limitar el ángulo vertical para evitar que la cámara se voltee
        self.vertical_angle = self.vertical_angle.clamp(-1.5, 1.5);
        
        self.update_position();
    }

    pub fn zoom(&mut self, delta: f32) {
        self.distance_from_target += delta;
        self.distance_from_target = self.distance_from_target.clamp(200.0, 3000.0);
        self.update_position();
    }

    pub fn set_target(&mut self, new_target: Vec3) {
        self.target = new_target;
        self.update_position();
    }

    pub fn update_aspect_ratio(&mut self, width: f32, height: f32) {
        self.aspect_ratio = width / height;
    }
}