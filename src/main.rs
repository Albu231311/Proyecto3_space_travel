use nalgebra_glm::{Vec3, Mat4, Vec4, look_at, perspective};
use minifb::{Key, Window, WindowOptions, KeyRepeat};
use std::time::{Duration, Instant};
use std::f32::consts::PI;

mod framebuffer;
mod triangle;
mod vertex;
mod color;
mod fragment;
mod shaders;
mod noise;
mod planet_shaders;
mod obj;

use framebuffer::Framebuffer;
use vertex::Vertex;
use triangle::triangle;
use shaders::vertex_shader;
use obj::Obj;

pub struct Uniforms {
    pub model_matrix: Mat4,
    pub view_projection_matrix: Mat4,
    pub time: f32,
    pub current_shader: u32,
    pub is_moon: bool,
    pub screen_width: f32,
    pub screen_height: f32,
}

// Estructura para la cámara/nave espacial
struct SpaceshipCamera {
    position: Vec3,
    rotation: Vec3, // pitch, yaw, roll
    velocity: Vec3,
    fov: f32,
    aspect_ratio: f32,
}

impl SpaceshipCamera {
    fn new(width: f32, height: f32) -> Self {
        SpaceshipCamera {
            position: Vec3::new(0.0, 200.0, 1500.0), // Comenzar lejos para ver todo el sistema
            rotation: Vec3::new(0.0, 0.0, 0.0),
            velocity: Vec3::new(0.0, 0.0, 0.0),
            fov: 60.0_f32.to_radians(),
            aspect_ratio: width / height,
        }
    }

    fn get_view_matrix(&self) -> Mat4 {
        // La cámara está detrás de la nave, mirando hacia la nave
        let camera_position = self.get_camera_position();
        let camera_target = self.get_camera_target();
        let up = Vec3::new(0.0, 1.0, 0.0);
        
        look_at(&camera_position, &camera_target, &up)
    }

    fn get_projection_matrix(&self) -> Mat4 {
        perspective(self.fov, self.aspect_ratio, 1.0, 10000.0)
    }

    fn get_view_projection_matrix(&self) -> Mat4 {
        self.get_projection_matrix() * self.get_view_matrix()
    }

    fn get_forward_vector(&self) -> Vec3 {
        let (sin_pitch, cos_pitch) = self.rotation.x.sin_cos();
        let (sin_yaw, cos_yaw) = self.rotation.y.sin_cos();
        
        Vec3::new(
            cos_pitch * sin_yaw,
            -sin_pitch,
            cos_pitch * cos_yaw,
        )
    }

    fn get_right_vector(&self) -> Vec3 {
        let yaw = self.rotation.y;
        Vec3::new(yaw.cos(), 0.0, -yaw.sin())
    }

    fn update(&mut self, dt: f32) {
        // Actualizar posición con velocidad
        self.position += self.velocity * dt;
        
        // Damping para que la nave se detenga gradualmente
        self.velocity *= 0.95;
    }

    fn get_spaceship_transform(&self) -> Mat4 {
        // La nave está exactamente en self.position
        create_model_matrix(self.position, 10.0, self.rotation)
    }
    
    fn get_camera_position(&self) -> Vec3 {
        // La cámara está detrás de la nave para vista de tercera persona
        let backward = -self.get_forward_vector(); // Vector hacia atrás
        let up = Vec3::new(0.0, 1.0, 0.0);
        
        // Posicionar cámara atrás y un poco arriba de la nave
        self.position + backward * 100.0 + up * 40.0
    }
    
    fn get_camera_target(&self) -> Vec3 {
        // La cámara mira hacia la nave
        self.position
    }

    // Nueva función para obtener la dirección de vista para el skybox
    fn get_view_direction(&self) -> Vec3 {
        let camera_pos = self.get_camera_position();
        let camera_target = self.get_camera_target();
        (camera_target - camera_pos).normalize()
    }
}

// Estructura para definir cada planeta
struct Planet {
    shader_id: u32,
    scale: f32,
    orbit_radius: f32,
    orbit_speed: f32,
    rotation_speed: f32,
    has_moon: bool,
}

fn create_model_matrix(translation: Vec3, scale: f32, rotation: Vec3) -> Mat4 {
    let (sin_x, cos_x) = rotation.x.sin_cos();
    let (sin_y, cos_y) = rotation.y.sin_cos();
    let (sin_z, cos_z) = rotation.z.sin_cos();

    let rotation_matrix_x = Mat4::new(
        1.0,  0.0,    0.0,   0.0,
        0.0,  cos_x, -sin_x, 0.0,
        0.0,  sin_x,  cos_x, 0.0,
        0.0,  0.0,    0.0,   1.0,
    );

    let rotation_matrix_y = Mat4::new(
        cos_y,  0.0,  sin_y, 0.0,
        0.0,    1.0,  0.0,   0.0,
        -sin_y, 0.0,  cos_y, 0.0,
        0.0,    0.0,  0.0,   1.0,
    );

    let rotation_matrix_z = Mat4::new(
        cos_z, -sin_z, 0.0, 0.0,
        sin_z,  cos_z, 0.0, 0.0,
        0.0,    0.0,  1.0, 0.0,
        0.0,    0.0,  0.0, 1.0,
    );

    let rotation_matrix = rotation_matrix_z * rotation_matrix_y * rotation_matrix_x;

    let scale_matrix = Mat4::new(
        scale, 0.0,   0.0,   0.0,
        0.0,   scale, 0.0,   0.0,
        0.0,   0.0,   scale, 0.0,
        0.0,   0.0,   0.0,   1.0,
    );

    let translation_matrix = Mat4::new(
        1.0, 0.0, 0.0, translation.x,
        0.0, 1.0, 0.0, translation.y,
        0.0, 0.0, 1.0, translation.z,
        0.0, 0.0, 0.0, 1.0,
    );

    translation_matrix * rotation_matrix * scale_matrix
}

fn create_sphere(radius: f32, segments: u32) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for lat in 0..=segments {
        let theta = lat as f32 * PI / segments as f32;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        for lon in 0..=segments {
            let phi = lon as f32 * 2.0 * PI / segments as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            let x = radius * sin_theta * cos_phi;
            let y = radius * cos_theta;
            let z = radius * sin_theta * sin_phi;

            let position = Vec3::new(x, y, z);
            let normal = position.normalize();

            vertices.push(Vertex::new(position, normal, nalgebra_glm::Vec2::new(0.0, 0.0)));
        }
    }

    for lat in 0..segments {
        for lon in 0..segments {
            let first = lat * (segments + 1) + lon;
            let second = first + segments + 1;

            indices.push(first);
            indices.push(second);
            indices.push(first + 1);

            indices.push(second);
            indices.push(second + 1);
            indices.push(first + 1);
        }
    }

    (vertices, indices)
}

// ===== FUNCIONES DEL SKYBOX CON ESTRELLAS =====

// Función de hash para generar números pseudo-aleatorios
fn hash(seed: u32) -> u32 {
    let mut x = seed;
    x ^= x >> 16;
    x = x.wrapping_mul(0x45d9f3b);
    x ^= x >> 16;
    x = x.wrapping_mul(0x45d9f3b);
    x ^= x >> 16;
    x
}

// Función para obtener coordenadas de pixel de una dirección 3D
fn direction_to_pixel(dir: Vec3, width: f32, height: f32) -> (i32, i32) {
    // Convertir dirección 3D a coordenadas esféricas
    let phi = dir.y.asin(); // Latitud
    let theta = dir.z.atan2(dir.x); // Longitud
    
    // Convertir a coordenadas de pantalla
    let u = (theta + PI) / (2.0 * PI); // 0 a 1
    let v = (phi + PI / 2.0) / PI; // 0 a 1
    
    let x = (u * width) as i32;
    let y = ((1.0 - v) * height) as i32; // Invertir Y
    
    (x.clamp(0, width as i32 - 1), y.clamp(0, height as i32 - 1))
}

// Renderizar skybox con estrellas
fn render_starfield_skybox(framebuffer: &mut Framebuffer, camera: &SpaceshipCamera) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let fov_half = camera.fov * 0.5;
    
    // Generar estrellas usando hash determinístico
    let star_count = 800; // Cantidad de estrellas
    
    for i in 0..star_count {
        // Generar posición de estrella usando hash
        let seed = i as u32 * 1000 + 12345;
        let hash1 = hash(seed);
        let hash2 = hash(seed + 1);
        let hash3 = hash(seed + 2);
        let hash4 = hash(seed + 3);
        
        // Convertir hash a coordenadas esféricas uniformes
        let theta = (hash1 as f32 / u32::MAX as f32) * 2.0 * PI; // 0 a 2π
        let phi = ((hash2 as f32 / u32::MAX as f32) * 2.0 - 1.0).acos(); // 0 a π
        
        // Convertir a dirección 3D
        let star_dir = Vec3::new(
            phi.sin() * theta.cos(),
            phi.cos(),
            phi.sin() * theta.sin()
        );
        
        // Obtener dirección de vista de la cámara
        let view_dir = camera.get_view_direction();
        let up = Vec3::new(0.0, 1.0, 0.0);
        let right = view_dir.cross(&up).normalize();
        let camera_up = right.cross(&view_dir).normalize();
        
        // Transformar estrella al espacio de pantalla
        let star_in_view = Vec3::new(
            star_dir.dot(&right),
            star_dir.dot(&camera_up),
            star_dir.dot(&view_dir)
        );
        
        // Solo dibujar si la estrella está enfrente de la cámara
        if star_in_view.z > 0.1 {
            // Proyección a pantalla
            let screen_x = (star_in_view.x / star_in_view.z / fov_half.tan() * 0.5 + 0.5) * width;
            let screen_y = (-star_in_view.y / star_in_view.z / fov_half.tan() * 0.5 + 0.5) * height;
            
            let pixel_x = screen_x as i32;
            let pixel_y = screen_y as i32;
            
            // Verificar que esté dentro de la pantalla
            if pixel_x >= 0 && pixel_x < width as i32 && pixel_y >= 0 && pixel_y < height as i32 {
                // Determinar brillo de la estrella
                let brightness = (hash3 as f32 / u32::MAX as f32) * 0.8 + 0.2; // 0.2 a 1.0
                let size = if hash4 % 100 < 5 { 2 } else { 1 }; // 5% de estrellas grandes
                
                // Color de la estrella
                let star_brightness = (brightness * 255.0) as u8;
                let star_color = if hash4 % 100 < 10 {
                    // 10% estrellas azuladas (estrellas jóvenes)
                    ((150 + star_brightness / 2) as u32) << 16 | 
                    ((180 + star_brightness / 3) as u32) << 8 | 
                    (star_brightness as u32)
                } else if hash4 % 100 < 20 {
                    // 10% estrellas rojizas (estrellas viejas)
                    (star_brightness as u32) << 16 | 
                    ((star_brightness / 2) as u32) << 8 | 
                    ((star_brightness / 3) as u32)
                } else {
                    // 80% estrellas blancas
                    (star_brightness as u32) << 16 | 
                    (star_brightness as u32) << 8 | 
                    (star_brightness as u32)
                };
                
                framebuffer.set_current_color(star_color);
                
                // Dibujar estrella
                for dy in 0..size {
                    for dx in 0..size {
                        let x = pixel_x + dx as i32;
                        let y = pixel_y + dy as i32;
                        if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                            framebuffer.point(x as usize, y as usize, 1.0); // Profundidad máxima
                        }
                    }
                }
            }
        }
    }
}

fn world_to_screen(world_pos: Vec3, view_projection_matrix: Mat4, screen_width: f32, screen_height: f32) -> Option<Vec3> {
    let world_position = Vec4::new(world_pos.x, world_pos.y, world_pos.z, 1.0);
    let projected = view_projection_matrix * world_position;
    
    let w = projected.w;
    if w <= 0.0 {
        return None;
    }
    
    let ndc_x = projected.x / w;
    let ndc_y = projected.y / w;
    let ndc_z = projected.z / w;
    
    if ndc_x < -1.0 || ndc_x > 1.0 || ndc_y < -1.0 || ndc_y > 1.0 {
        return None;
    }
    
    let screen_x = (ndc_x + 1.0) * 0.5 * screen_width;
    let screen_y = (1.0 - ndc_y) * 0.5 * screen_height;
    
    Some(Vec3::new(screen_x, screen_y, ndc_z))
}

fn draw_line(framebuffer: &mut Framebuffer, x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
    let mut x0 = x0;
    let mut y0 = y0;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    framebuffer.set_current_color(color);
    
    loop {
        if x0 >= 0 && x0 < framebuffer.width as i32 && y0 >= 0 && y0 < framebuffer.height as i32 {
            framebuffer.point(x0 as usize, y0 as usize, 0.999);
        }
        
        if x0 == x1 && y0 == y1 {
            break;
        }
        
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

fn render_orbit_line(framebuffer: &mut Framebuffer, center: Vec3, radius: f32, view_projection_matrix: Mat4, screen_width: f32, screen_height: f32) {
    let segments = 128;
    let orbit_color = 0x333333;
    
    for i in 0..segments {
        let angle1 = (i as f32 / segments as f32) * 2.0 * PI;
        let angle2 = ((i + 1) % segments) as f32 / segments as f32 * 2.0 * PI;
        
        let point1 = Vec3::new(
            center.x + radius * angle1.cos(),
            center.y,
            center.z + radius * angle1.sin()
        );
        
        let point2 = Vec3::new(
            center.x + radius * angle2.cos(),
            center.y,
            center.z + radius * angle2.sin()
        );
        
        let screen_point1 = world_to_screen(point1, view_projection_matrix, screen_width, screen_height);
        let screen_point2 = world_to_screen(point2, view_projection_matrix, screen_width, screen_height);
        
        if let (Some(sp1), Some(sp2)) = (screen_point1, screen_point2) {
            draw_line(framebuffer, sp1.x as i32, sp1.y as i32, sp2.x as i32, sp2.y as i32, orbit_color);
        }
    }
}

fn render_debug_spaceship_cube(framebuffer: &mut Framebuffer, camera: &SpaceshipCamera, view_projection_matrix: Mat4, screen_width: f32, screen_height: f32) {
    // Crear un cubo simple como nave de depuración
    let cube_vertices = vec![
        // Frente
        Vertex::new(Vec3::new(-1.0, -1.0,  1.0), Vec3::new(0.0, 0.0, 1.0), nalgebra_glm::Vec2::new(0.0, 0.0)),
        Vertex::new(Vec3::new( 1.0, -1.0,  1.0), Vec3::new(0.0, 0.0, 1.0), nalgebra_glm::Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new( 1.0,  1.0,  1.0), Vec3::new(0.0, 0.0, 1.0), nalgebra_glm::Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new(-1.0,  1.0,  1.0), Vec3::new(0.0, 0.0, 1.0), nalgebra_glm::Vec2::new(0.0, 1.0)),
        // Atrás
        Vertex::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(0.0, 0.0, -1.0), nalgebra_glm::Vec2::new(1.0, 0.0)),
        Vertex::new(Vec3::new(-1.0,  1.0, -1.0), Vec3::new(0.0, 0.0, -1.0), nalgebra_glm::Vec2::new(1.0, 1.0)),
        Vertex::new(Vec3::new( 1.0,  1.0, -1.0), Vec3::new(0.0, 0.0, -1.0), nalgebra_glm::Vec2::new(0.0, 1.0)),
        Vertex::new(Vec3::new( 1.0, -1.0, -1.0), Vec3::new(0.0, 0.0, -1.0), nalgebra_glm::Vec2::new(0.0, 0.0)),
    ];
    
    let cube_indices = vec![
        0, 1, 2,  2, 3, 0,  // frente
        4, 5, 6,  6, 7, 4,  // atrás
        3, 2, 6,  6, 5, 3,  // arriba
        0, 4, 7,  7, 1, 0,  // abajo
        0, 3, 5,  5, 4, 0,  // izquierda
        1, 7, 6,  6, 2, 1,  // derecha
    ];
    
    let cube_model_matrix = camera.get_spaceship_transform();
    let cube_uniforms = Uniforms {
        model_matrix: cube_model_matrix,
        view_projection_matrix,
        time: 0.0,
        current_shader: 6, // Shader de nave
        is_moon: false,
        screen_width,
        screen_height,
    };
    
    render_object(framebuffer, &cube_uniforms, &cube_vertices, &cube_indices);
}

fn render_object(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertices: &[Vertex], indices: &[u32]) {
    let mut transformed_vertices = Vec::with_capacity(vertices.len());
    for vertex in vertices {
        let transformed = vertex_shader(vertex, uniforms);
        transformed_vertices.push(transformed);
    }

    for triangle_idx in (0..indices.len()).step_by(3) {
        if triangle_idx + 2 < indices.len() {
            let i1 = indices[triangle_idx] as usize;
            let i2 = indices[triangle_idx + 1] as usize;
            let i3 = indices[triangle_idx + 2] as usize;

            if i1 < transformed_vertices.len() && i2 < transformed_vertices.len() && i3 < transformed_vertices.len() {
                let v1 = &transformed_vertices[i1];
                let v2 = &transformed_vertices[i2];
                let v3 = &transformed_vertices[i3];

                let fragments = triangle(v1, v2, v3, uniforms);
                
                for fragment in fragments {
                    let x = fragment.position.x as usize;
                    let y = fragment.position.y as usize;
                    if x < framebuffer.width && y < framebuffer.height {
                        let color = fragment.color.to_hex();
                        framebuffer.set_current_color(color);
                        framebuffer.point(x, y, fragment.depth);
                    }
                }
            }
        }
    }
}

fn main() {
    let window_width = 1200;
    let window_height = 800;
    let start_time = Instant::now();

    let mut framebuffer = Framebuffer::new(window_width, window_height);
    let mut window = Window::new(
        "Sistema Solar 3D con Skybox de Estrellas ",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();

    // Crear cámara/nave espacial
    let mut spaceship_camera = SpaceshipCamera::new(window_width as f32, window_height as f32);
    
    framebuffer.set_background_color(0x000008);

    // Cargar modelo de la nave espacial
    let spaceship_obj = match Obj::load("assets/nave2.obj") {
        Ok(obj) => {
            println!("Nave espacial cargada exitosamente!");
            Some(obj)
        },
        Err(e) => {
            println!("No se pudo cargar la nave espacial (assets/nave2.obj): {:?}", e);
            println!("Continuando con cubo de depuración...");
            None
        }
    };

    let (spaceship_vertices, spaceship_indices) = if let Some(ref obj) = spaceship_obj {
        obj.get_vertex_and_index_arrays()
    } else {
        (Vec::new(), Vec::new())
    };

    // Definir planetas con escalas más grandes para exploración
    let planets = vec![
        Planet { shader_id: 4, scale: 50.0, orbit_radius: 300.0, orbit_speed: 1.6, rotation_speed: 0.02, has_moon: false },
        Planet { shader_id: 2, scale: 80.0, orbit_radius: 500.0, orbit_speed: 1.2, rotation_speed: 0.015, has_moon: false },
        Planet { shader_id: 0, scale: 85.0, orbit_radius: 700.0, orbit_speed: 1.0, rotation_speed: 0.05, has_moon: true },
        Planet { shader_id: 3, scale: 70.0, orbit_radius: 900.0, orbit_speed: 0.8, rotation_speed: 0.048, has_moon: false },
        Planet { shader_id: 2, scale: 150.0, orbit_radius: 1200.0, orbit_speed: 0.4, rotation_speed: 0.1, has_moon: false },
        Planet { shader_id: 5, scale: 130.0, orbit_radius: 1500.0, orbit_speed: 0.3, rotation_speed: 0.09, has_moon: false },
        Planet { shader_id: 5, scale: 100.0, orbit_radius: 1800.0, orbit_speed: 0.2, rotation_speed: 0.07, has_moon: false },
        Planet { shader_id: 5, scale: 95.0, orbit_radius: 2100.0, orbit_speed: 0.15, rotation_speed: 0.065, has_moon: false },
    ];

    // Generar esfera para planetas
    let (planet_vertices, planet_indices) = create_sphere(1.0, 20);

    let mut global_speed = 1.0f32;
    let mut paused = false;
    let mut show_orbits = true;

    println!("   Controles de la Nave Espacial:");
    println!("   WASD: Mover nave");
    println!("   Flechas: Rotar nave");
    println!("   Q/E: Roll");
    println!("   Shift: Turbo");
    println!("   O: Mostrar/Ocultar órbitas");
    println!("   +/-: Velocidad del sistema");
    println!("   ESC: Salir");

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        handle_input(&window, &mut spaceship_camera, &mut global_speed, &mut paused, &mut show_orbits);

        let elapsed = if paused {
            start_time.elapsed().as_secs_f32() - 0.016
        } else {
            start_time.elapsed().as_secs_f32() * global_speed
        };

        spaceship_camera.update(0.016);
        framebuffer.clear();

        // === RENDERIZAR SKYBOX DE ESTRELLAS PRIMERO ===
        render_starfield_skybox(&mut framebuffer, &spaceship_camera);

        let view_projection_matrix = spaceship_camera.get_view_projection_matrix();

        // === RENDERIZAR ÓRBITAS ===
        if show_orbits {
            let orbit_center = Vec3::new(0.0, 0.0, 0.0);
            for planet in &planets {
                render_orbit_line(
                    &mut framebuffer, 
                    orbit_center, 
                    planet.orbit_radius, 
                    view_projection_matrix, 
                    window_width as f32, 
                    window_height as f32
                );
            }
        }

        // === RENDERIZAR EL SOL ===
        let sun_scale = 200.0;
        let sun_rotation = Vec3::new(0.0, elapsed * 0.1, 0.0);
        let sun_model_matrix = create_model_matrix(Vec3::new(0.0, 0.0, 0.0), sun_scale, sun_rotation);
        let sun_uniforms = Uniforms {
            model_matrix: sun_model_matrix,
            view_projection_matrix,
            time: elapsed,
            current_shader: 1,
            is_moon: false,
            screen_width: window_width as f32,
            screen_height: window_height as f32,
        };
        render_object(&mut framebuffer, &sun_uniforms, &planet_vertices, &planet_indices);

        // === RENDERIZAR PLANETAS ===
        for planet in &planets {
            let orbit_angle = elapsed * planet.orbit_speed;
            let planet_x = planet.orbit_radius * orbit_angle.cos();
            let planet_z = planet.orbit_radius * orbit_angle.sin();
            let planet_y = 0.0;

            let planet_rotation = Vec3::new(0.0, elapsed * planet.rotation_speed, 0.0);
            let planet_model_matrix = create_model_matrix(
                Vec3::new(planet_x, planet_y, planet_z),
                planet.scale,
                planet_rotation
            );

            let planet_uniforms = Uniforms {
                model_matrix: planet_model_matrix,
                view_projection_matrix,
                time: elapsed,
                current_shader: planet.shader_id,
                is_moon: false,
                screen_width: window_width as f32,
                screen_height: window_height as f32,
            };

            render_object(&mut framebuffer, &planet_uniforms, &planet_vertices, &planet_indices);

            // Luna para la Tierra
            if planet.has_moon {
                let moon_scale = planet.scale * 0.27;
                let moon_orbit_radius = planet.scale * 2.0;
                let moon_angle = elapsed * 2.0;
                let moon_x = planet_x + moon_orbit_radius * moon_angle.cos();
                let moon_z = planet_z + moon_orbit_radius * moon_angle.sin();
                let moon_y = planet_y;

                let moon_model_matrix = create_model_matrix(
                    Vec3::new(moon_x, moon_y, moon_z),
                    moon_scale,
                    Vec3::new(0.0, elapsed * 0.5, 0.0)
                );

                let moon_uniforms = Uniforms {
                    model_matrix: moon_model_matrix,
                    view_projection_matrix,
                    time: elapsed,
                    current_shader: 0,
                    is_moon: true,
                    screen_width: window_width as f32,
                    screen_height: window_height as f32,
                };

                render_object(&mut framebuffer, &moon_uniforms, &planet_vertices, &planet_indices);
            }
        }

        // === RENDERIZAR LA NAVE ESPACIAL ===
        if !spaceship_vertices.is_empty() {
            let spaceship_model_matrix = spaceship_camera.get_spaceship_transform();
            let spaceship_uniforms = Uniforms {
                model_matrix: spaceship_model_matrix,
                view_projection_matrix,
                time: elapsed,
                current_shader: 6, // Shader especial para la nave
                is_moon: false,
                screen_width: window_width as f32,
                screen_height: window_height as f32,
            };

            render_object(&mut framebuffer, &spaceship_uniforms, &spaceship_vertices, &spaceship_indices);
        } else {
            // Si no hay nave, renderizar un cubo simple como indicador
            render_debug_spaceship_cube(&mut framebuffer, &spaceship_camera, view_projection_matrix, window_width as f32, window_height as f32);
        }

        window
            .update_with_buffer(&framebuffer.buffer, window_width, window_height)
            .unwrap();

        std::thread::sleep(Duration::from_millis(16));
    }
}

fn handle_input(window: &Window, camera: &mut SpaceshipCamera, global_speed: &mut f32, paused: &mut bool, show_orbits: &mut bool) {
    let base_speed = 8.0; // Un poco más rápido para mejor navegación
    let turbo_multiplier = if window.is_key_down(Key::LeftShift) { 3.0 } else { 1.0 };
    let move_speed = base_speed * turbo_multiplier;
    let rotation_speed = 0.03; // Un poco más responsivo

    // === MOVIMIENTO DE NAVE ===
    if window.is_key_down(Key::W) {
        camera.velocity += camera.get_forward_vector() * move_speed;
    }
    if window.is_key_down(Key::S) {
        camera.velocity -= camera.get_forward_vector() * move_speed;
    }
    if window.is_key_down(Key::A) {
        camera.velocity -= camera.get_right_vector() * move_speed;
    }
    if window.is_key_down(Key::D) {
        camera.velocity += camera.get_right_vector() * move_speed;
    }

    // === ROTACIÓN ===
    if window.is_key_down(Key::Up) {
        camera.rotation.x += rotation_speed;
    }
    if window.is_key_down(Key::Down) {
        camera.rotation.x -= rotation_speed;
    }
    if window.is_key_down(Key::Left) {
        camera.rotation.y += rotation_speed;
    }
    if window.is_key_down(Key::Right) {
        camera.rotation.y -= rotation_speed;
    }
    if window.is_key_down(Key::Q) {
        camera.rotation.z += rotation_speed;
    }
    if window.is_key_down(Key::E) {
        camera.rotation.z -= rotation_speed;
    }

    // Limitar pitch
    camera.rotation.x = camera.rotation.x.clamp(-PI / 2.5, PI / 2.5);

    // === CONTROLES DEL SISTEMA ===
    if window.is_key_pressed(Key::Space, KeyRepeat::No) {
        *paused = !*paused;
        println!("{}", if *paused { "Sistema PAUSADO" } else { "▶Sistema REANUDADO" });
    }

    if window.is_key_pressed(Key::O, KeyRepeat::No) {
        *show_orbits = !*show_orbits;
        println!("{}", if *show_orbits { "Órbitas VISIBLES" } else { "Órbitas OCULTAS" });
    }

    if window.is_key_down(Key::Equal) {
        *global_speed += 0.1;
        println!("⚡ Velocidad del sistema: {:.1}x", global_speed);
    }
    if window.is_key_down(Key::Minus) {
        *global_speed = (*global_speed - 0.1).max(0.1);
        println!("Velocidad del sistema: {:.1}x", global_speed);
    }
}