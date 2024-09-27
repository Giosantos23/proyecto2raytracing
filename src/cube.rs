use nalgebra_glm::{Vec3};
use crate::ray_intersect::{RayIntersect, Intersect};
use crate::material::Material;

pub struct Cube {
    pub min: Vec3,       // Punto mínimo del cubo (esquina inferior izquierda)
    pub max: Vec3,       // Punto máximo del cubo (esquina superior derecha)
    pub material: Material,
}

impl Cube {
    pub fn new(min: Vec3, max: Vec3, material: Material) -> Self {
        Self { min, max, material }
    }
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        let mut t_min = (self.min.x - ray_origin.x) / ray_direction.x;
        let mut t_max = (self.max.x - ray_origin.x) / ray_direction.x;

        if t_min > t_max {
            std::mem::swap(&mut t_min, &mut t_max);
        }

        let mut ty_min = (self.min.y - ray_origin.y) / ray_direction.y;
        let mut ty_max = (self.max.y - ray_origin.y) / ray_direction.y;

        if ty_min > ty_max {
            std::mem::swap(&mut ty_min, &mut ty_max);
        }

        if (t_min > ty_max) || (ty_min > t_max) {
            return Intersect::empty();  // No hay intersección
        }

        if ty_min > t_min {
            t_min = ty_min;
        }
        if ty_max < t_max {
            t_max = ty_max;
        }

        let mut tz_min = (self.min.z - ray_origin.z) / ray_direction.z;
        let mut tz_max = (self.max.z - ray_origin.z) / ray_direction.z;

        if tz_min > tz_max {
            std::mem::swap(&mut tz_min, &mut tz_max);
        }

        if (t_min > tz_max) || (tz_min > t_max) {
            return Intersect::empty();  // No hay intersección
        }

        if tz_min > t_min {
            t_min = tz_min;
        }
        if tz_max < t_max {
            t_max = tz_max;
        }

        let intersection_distance = t_min;
        if intersection_distance < 0.0 {
            return Intersect::empty();  // La intersección está detrás de la cámara
        }

        let intersect_point = ray_origin + ray_direction * intersection_distance;
        let normal = self.compute_normal(&intersect_point);

        // Convierte el punto de intersección a coordenadas UV


        Intersect {
            point: intersect_point,
            normal,
            distance: intersection_distance,
            material: self.material.clone(),  // Asumimos que la textura está en el material
            is_intersecting: true,
        }
    }
}

impl Cube {
    fn compute_normal(&self, point: &Vec3) -> Vec3 {
        if (point.x - self.min.x).abs() < 1e-4 {
            Vec3::new(-1.0, 0.0, 0.0)
        } else if (point.x - self.max.x).abs() < 1e-4 {
            Vec3::new(1.0, 0.0, 0.0)
        } else if (point.y - self.min.y).abs() < 1e-4 {
            Vec3::new(0.0, -1.0, 0.0)
        } else if (point.y - self.max.y).abs() < 1e-4 {
            Vec3::new(0.0, 1.0, 0.0)
        } else if (point.z - self.min.z).abs() < 1e-4 {
            Vec3::new(0.0, 0.0, -1.0)
        } else {
            Vec3::new(0.0, 0.0, 1.0)
        }
    }
    
    // Convierte el punto de intersección 3D a coordenadas UV


}

// Crear tronco
pub fn create_tronco(base: Vec3, top: Vec3, material: Material) -> Box<Cube> {
    Box::new(Cube::new(base, top, material))
}