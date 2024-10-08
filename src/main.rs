mod framebuffer;
mod ray_intersect;
mod color;
mod camera;
mod light;
mod material;
mod cube; 
mod grid;
mod group;
mod texture;


use minifb::{ Window, WindowOptions, Key };
use nalgebra_glm::{Vec3, normalize};
use std::f32::consts::PI;
use std::time::{Duration, Instant};


use crate::color::Color;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::cube::create_tronco;


use crate::framebuffer::Framebuffer;
use crate::camera::Camera;
use crate::light::Light;
use crate::material::Material;
use crate::texture::Texture; 


use crate::grid::Grid;
use crate::group::Group;


const ORIGIN_BIAS: f32 = 1e-4;
const SKYBOX_COLOR: Color = Color::new(102, 153, 255);


fn offset_origin(intersect: &Intersect, direction: &Vec3) -> Vec3 {
    let offset = intersect.normal * ORIGIN_BIAS;
    if direction.dot(&intersect.normal) < 0.0 {
        intersect.point - offset
    } else {
        intersect.point + offset
    }
}

fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - 2.0 * incident.dot(normal) * normal
}

fn refract(incident: &Vec3, normal: &Vec3, eta_t: f32) -> Vec3 {
    let cosi = -incident.dot(normal).max(-1.0).min(1.0);
    
    let (n_cosi, eta, n_normal);

    if cosi < 0.0 {
        n_cosi = -cosi;
        eta = 1.0 / eta_t;
        n_normal = -normal;
    } else {
        n_cosi = cosi;
        eta = eta_t;
        n_normal = *normal;
    }
    
    let k = 1.0 - eta * eta * (1.0 - n_cosi * n_cosi);
    
    if k < 0.0 {
        reflect(incident, &n_normal)
    } else {
        eta * incident + (eta * n_cosi - k.sqrt()) * n_normal
    }
}

fn cast_shadow(
    intersect: &Intersect,
    light: &Light,
    objects: &[Box<dyn RayIntersect>],  
) -> f32 {
    let light_dir = (light.position - intersect.point).normalize();
    let light_distance = (light.position - intersect.point).magnitude();

    let shadow_ray_origin = offset_origin(intersect, &light_dir);
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
    objects: &[Box<dyn RayIntersect>], 
    light: &Light,
    depth: u32,
) -> Color {
    if depth > 3 {
        return SKYBOX_COLOR;
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
        return SKYBOX_COLOR;
    }

    let light_dir = (light.position - intersect.point).normalize();
    let view_dir = (ray_origin - intersect.point).normalize();
    let reflect_dir = reflect(&-light_dir, &intersect.normal).normalize();

    let shadow_intensity = cast_shadow(&intersect, light, objects);
    let light_intensity = light.intensity * (1.0 - shadow_intensity);

    let diffuse_intensity = intersect.normal.dot(&light_dir).max(0.0).min(1.0);
    let diffuse = intersect.material.diffuse * intersect.material.albedo[0] * diffuse_intensity * light_intensity;

    let specular_intensity = view_dir.dot(&reflect_dir).max(0.0).powf(intersect.material.specular);
    let specular = light.color * intersect.material.albedo[1] * specular_intensity * light_intensity;

    let mut reflect_color = Color::black();
    let reflectivity = intersect.material.albedo[2];
    if reflectivity > 0.0 {
        let reflect_dir = reflect(&ray_direction, &intersect.normal).normalize();
        let reflect_origin = offset_origin(&intersect, &reflect_dir);
        reflect_color = cast_ray(&reflect_origin, &reflect_dir, objects, light, depth + 1);
    }


    let mut refract_color = Color::black();
    let transparency = intersect.material.albedo[3];
    if transparency > 0.0 {
        let refract_dir = refract(&ray_direction, &intersect.normal, intersect.material.refractive_index);
        let refract_origin = offset_origin(&intersect, &refract_dir);
        refract_color = cast_ray(&refract_origin, &refract_dir, objects, light, depth + 1);
    }

    (diffuse + specular) * (1.0 - reflectivity - transparency) + (reflect_color * reflectivity) + (refract_color * transparency)
}

pub fn render(framebuffer: &mut Framebuffer, objects: &[Box<dyn RayIntersect>], camera: &Camera, light: &Light) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI/3.0;
    let perspective_scale = (fov * 0.5).tan();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            let ray_direction = normalize(&Vec3::new(screen_x, screen_y, -1.0));

            let rotated_direction = camera.base_change(&ray_direction);

            let pixel_color = cast_ray(&camera.eye, &rotated_direction, objects, light, 0);

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
        "Refractor",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();


    let agua = Material::new(
        Color::new(0, 255, 255), // Color del piso
        500.0, // Especularidad
        [0.1, 0.5, 0.3, 0.8], // Albedo
        1.33,  // Índice de refracción
        None,
    );
    let textarena = Some(Texture::from_file("arena.png"));
    let arena = Material::new(
        Color::new(203, 189, 147), 
        1000.0, 
        [0.9, 0.5, 0.1, 0.0], 
        1.0,
        textarena,
    );
    let textpalmera = Some(Texture::from_file("palmeratext.png"));
    let madera = Material::new(
        Color::new(161, 102, 47), 
        500.0, 
        [0.9, 0.4, 0.1, 0.0], 
        1.0,
        textpalmera,

    );
    let hoja_palmera = Material::new(
        Color::new(113, 178, 39), 
        0.9, 
        [0.7, 0.1, 0.1, 0.0], 
        1.0,
        None,
    );
    
    let oceano = Grid::create_cuadricula(6, 5, 0.3, agua.clone(),0.0, 0.0,0.0); 
    let oceano2 = Grid::create_cuadricula(4, 5, 0.3, agua.clone(), 11.0 * 0.3,0.0, 0.0); 
    let sand = Grid::create_cuadricula(6, 5, 0.3, arena.clone(), 5.0 * 0.3, 0.0, 0.0); 


//tronco palmera
    pub fn create_palmera(base_position: Vec3, material: Material) -> Group {
        let tronco1 = create_tronco(
            base_position + Vec3::new(3.0, 0.0, 0.0),  
            base_position + Vec3::new(3.1, 0.4, 0.2),  
            material.clone()
        );
        
        let tronco2 = create_tronco(
            base_position + Vec3::new(3.0, 0.4, 0.0),  
            base_position + Vec3::new(3.1, 0.8, 0.2),  
            material.clone()
        );   
    
        let tronco3 = create_tronco(
            base_position + Vec3::new(3.0, 0.8, 0.0),  
            base_position + Vec3::new(3.1, 1.2, 0.2),  
            material.clone()
        );    
    
        let mut palmera = Group::new(vec![tronco1, tronco2, tronco3], Vec3::new(0.0, 0.0, 0.0));
        palmera.set_offset(base_position);
        palmera
    }

    let palmera1 = create_palmera(Vec3::new(-0.3, 0.0, 0.6), madera.clone());
    let palmera2 = create_palmera(Vec3::new(-0.3, 0.0, 0.3), madera.clone());
    let palmera3 = create_palmera(Vec3::new(-0.3, 0.0, 0.0), madera.clone());

    pub fn gen_hojas(base_position: Vec3, material: Material) -> Group {
        let hojas1 = create_tronco(
            base_position + Vec3::new(3.0, 1.2, 0.0),  
            base_position + Vec3::new(3.1, 1.4, 0.2),  
            material.clone()
        );
        let hojas2 = create_tronco(
            base_position + Vec3::new(3.1, 1.2, 0.0),  
            base_position + Vec3::new(3.2, 1.4, 0.2),  
            material.clone()
        );
        let hojas3 = create_tronco(
            base_position + Vec3::new(2.9, 1.2, 0.0),  
            base_position + Vec3::new(3.0, 1.4, 0.2),  
            material.clone()
        );
        let hojas4 = create_tronco(
            base_position + Vec3::new(3.0, 1.4, 0.0),  
            base_position + Vec3::new(3.1, 1.6, 0.2),  
            material.clone()
        );

        
    
        let mut hojas = Group::new(vec![hojas1,hojas2,hojas3,hojas4], Vec3::new(0.0, 0.0, 0.0));
        hojas.set_offset(base_position);
        hojas
    }
    let hoja1 = gen_hojas(Vec3::new(-0.3, 0.0, 0.6), hoja_palmera.clone());
    let hoja2 = gen_hojas(Vec3::new(-0.3, 0.0, 0.3), hoja_palmera.clone());
    let hoja3 = gen_hojas(Vec3::new(-0.3, 0.0, 0.0), hoja_palmera.clone());

    



    

    let objects: Vec<Box<dyn RayIntersect>> = vec![
        Box::new(oceano),
        Box::new(oceano2),
        Box::new(sand),
        Box::new(palmera1),
        Box::new(palmera2),
        Box::new(palmera3),
        Box::new(hoja1),
        Box::new(hoja2),
        Box::new(hoja3),


        ]; 



    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 5.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let mut light = Light::new(
        Vec3::new(0.0, 10.0, 0.0),
        Color::new(255, 255, 255),
        1.0
    );

    let rotation_speed = PI/10.0;

    let start_time = Instant::now(); 
    let cycle_duration = 60.0; 


    while window.is_open() && !window.is_key_down(Key::Escape) {

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

        let elapsed_time = start_time.elapsed().as_secs_f32();
        let time_in_cycle = elapsed_time % cycle_duration;
        let day_fraction = time_in_cycle / cycle_duration; 

        let sun_angle = day_fraction * 2.0 * PI; 
        let sun_radius = 10.0; 
        let sun_x = sun_radius * sun_angle.cos();
        let sun_y = sun_radius * sun_angle.sin();
        let sun_z = 0.0; 

        light.position = Vec3::new(sun_x, sun_y, sun_z);
        let day_color = Color::new(255, 255, 224); 
        let night_color = Color::new(25, 25, 112); 

        let light_color = if day_fraction < 0.5 {
            let t = day_fraction / 0.5;
            day_color.interpolate(&night_color, t)
        } else {
            let t = (day_fraction - 0.5) / 0.5;
            night_color.interpolate(&day_color, 1.0 - t)
        };

        light.color = light_color;

        light.intensity = if day_fraction < 0.5 {
            1.0 - day_fraction * 2.0
        } else {
            0.1
        };



        render(&mut framebuffer, &objects, &camera, &light);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}   