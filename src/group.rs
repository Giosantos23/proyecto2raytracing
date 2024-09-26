use nalgebra_glm::Vec3;
use crate::ray_intersect::{RayIntersect, Intersect};
pub struct Group {
    pub elements: Vec<Box<dyn RayIntersect>>,
    pub offset: Vec3, 
}

impl Group {
    pub fn new(elements: Vec<Box<dyn RayIntersect>>, offset: Vec3) -> Self {
        Group { elements, offset }
    }

    pub fn add(&mut self, element: Box<dyn RayIntersect>) {
        self.elements.push(element);
    }

    pub fn set_offset(&mut self, offset: Vec3) {
        self.offset = offset;
    }
}

impl RayIntersect for Group {
    fn ray_intersect(&self, origin: &Vec3, direction: &Vec3) -> Intersect {
        let mut closest_intersect = Intersect::empty();
        let mut min_distance = f32::INFINITY;

        for element in &self.elements {
            let adjusted_origin = origin - self.offset;
            let adjusted_direction = direction; 

            let intersect = element.ray_intersect(&adjusted_origin, &adjusted_direction);
            if intersect.is_intersecting && intersect.distance < min_distance {
                min_distance = intersect.distance;
                closest_intersect = intersect;
            }
        }

        closest_intersect
    }
}
