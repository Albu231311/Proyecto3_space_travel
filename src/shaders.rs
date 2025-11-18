use nalgebra_glm::{Vec3, Vec4, Mat3};
use crate::vertex::Vertex;
use crate::Uniforms;

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
    // Transformación del modelo (mundo)
    let world_position = Vec4::new(
        vertex.position.x,
        vertex.position.y,
        vertex.position.z,
        1.0
    );
    let world_transformed = uniforms.model_matrix * world_position;

    // Aplicar transformación de vista y proyección
    let projected = uniforms.view_projection_matrix * world_transformed;

    // Para proyección ortográfica, W debería ser 1.0, pero mantenemos la normalización por seguridad
    let w = projected.w.max(0.001);
    let ndc_x = projected.x / w;
    let ndc_y = projected.y / w;
    let ndc_z = projected.z / w;

    // Convertir NDC [-1,1] a coordenadas de pantalla [0, width/height]
    let screen_x = (ndc_x + 1.0) * 0.5 * uniforms.screen_width;
    let screen_y = (1.0 - ndc_y) * 0.5 * uniforms.screen_height; // Invertir Y
    let screen_z = (ndc_z + 1.0) * 0.5; // Normalizar depth a [0,1]

    let transformed_position = Vec3::new(screen_x, screen_y, screen_z);

    // Transformar las normales correctamente
    let model_mat3 = Mat3::new(
        uniforms.model_matrix[0], uniforms.model_matrix[1], uniforms.model_matrix[2],
        uniforms.model_matrix[4], uniforms.model_matrix[5], uniforms.model_matrix[6],
        uniforms.model_matrix[8], uniforms.model_matrix[9], uniforms.model_matrix[10]
    );
    let normal_matrix = model_mat3.transpose().try_inverse().unwrap_or(Mat3::identity());
    let transformed_normal = (normal_matrix * vertex.normal).normalize();

    Vertex {
        position: vertex.position,
        normal: vertex.normal,
        tex_coords: vertex.tex_coords,
        color: vertex.color,
        transformed_position,
        transformed_normal,
    }
}