use nalgebra_glm::Vec3;
use crate::ray_intersect::{RayIntersect, Intersect};
use crate::cube::Cube;  
use crate::material::Material;

pub struct Grid {
    pub objects: Vec<Box<dyn RayIntersect>>,
}

impl Grid {
    pub fn new(objects: Vec<Box<dyn RayIntersect>>) -> Self {
        Grid { objects }
    }

    // Función para crear una cuadrícula de cubos con un desplazamiento
    pub fn create_cuadricula(width: usize, depth: usize, cube_size: f32, material: Material, offset_x: f32, offset_y: f32, offset_z: f32) -> Self {
        let mut objects: Vec<Box<dyn RayIntersect>> = Vec::new();
    
        for x in 0..width {
            for z in 0..depth {
                // Calcula la posición de cada cubo con un offset
                let min = Vec3::new(
                    x as f32 * cube_size + offset_x, 
                    0.0 + offset_y,  // Aplica el desplazamiento en el eje Y
                    z as f32 * cube_size + offset_z
                );
    
                let max = min + Vec3::new(cube_size, cube_size, cube_size);
    
                let cube = Box::new(Cube::new(min, max, material.clone()));
                objects.push(cube);
            }
        }
    
        Grid::new(objects)
    }
    
// Función para crear una cuadrícula vertical girada 90 grados
// Función para crear una cuadrícula vertical girada 90 grados con desplazamiento en el eje X
    pub fn create_cuadricula_rotada(width: usize, height: usize, cube_size: f32, material: Material, offset_x: f32, offset_y: f32, offset_z: f32) -> Self {
        let mut objects: Vec<Box<dyn RayIntersect>> = Vec::new();

        for y in 0..height {
            for x in 0..width {
                // Calcula la posición de cada cubo con un offset
                let min = Vec3::new(
                    x as f32 * cube_size + offset_x,  // Desplaza los cubos en el eje X
                    y as f32 * cube_size + offset_y,  // Alinea los cubos en el eje Y
                    offset_z  // Mantén el Z fijo para la rotación de 90 grados
                );

                let max = min + Vec3::new(cube_size, cube_size, cube_size);

                let cube = Box::new(Cube::new(min, max, material.clone()));
                objects.push(cube);
            }
        }

        Grid::new(objects)
    }


}

impl RayIntersect for Grid {
    fn ray_intersect(&self, origin: &Vec3, direction: &Vec3) -> Intersect {
        let mut closest_intersect = Intersect::empty();
        let mut min_distance = f32::INFINITY;

        for object in &self.objects {
            let intersect = object.ray_intersect(origin, direction);
            if intersect.is_intersecting && intersect.distance < min_distance {
                min_distance = intersect.distance;
                closest_intersect = intersect;
            }
        }

        closest_intersect
    }
}
