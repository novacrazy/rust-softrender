use softrender::mesh::Vertex;
use softrender::render::{ScreenVertex, ClipVertex, Fragment};

use ::color::{Color, SRGB_GAMMA, decode_gamma, encode_gamma, aces_filmic_tonemap};
use ::mesh::VertexData;
use ::uniforms::{GlobalUniforms, Uniforms};

pub fn vertex_shader(vertex: &Vertex<VertexData>, global_uniforms: &GlobalUniforms) -> ClipVertex<Uniforms> {
    let GlobalUniforms { ref view, ref projection, ref model, ref model_inverse_transpose, .. } = *global_uniforms;
    let VertexData { normal, uv } = vertex.vertex_data;

    let world_position = model * vertex.position.to_homogeneous();

    let normal = (model_inverse_transpose * normal.to_homogeneous()).normalize();

    let clip_position = projection * view * world_position;

    // Return the clip-space position and any uniforms to interpolate and pass into the fragment shader
    ClipVertex::new(clip_position, Uniforms {
        position: world_position,
        normal: normal,
        uv: uv,
    })
}

// Simple Fresnel Schlick approximation
fn fresnel_schlick(cos_theta: f32, ior: f32) -> f32 {
    let f0 = ((1.0 - ior) / (1.0 + ior)).powi(2);

    f0 + (1.0 - f0) * (1.0 - cos_theta).powi(5)
}

fn saturate(value: f32) -> f32 {
    if value < 0.0 { 0.0 } else if value > 1.0 { 1.0 } else { value }
}

// GLSL habits die hard
#[allow(non_snake_case)]
pub fn fragment_shader(vertex: &ScreenVertex<Uniforms>, global_uniforms: &GlobalUniforms) -> Fragment<Color> {
    let GlobalUniforms { ref camera, ref lights, .. } = *global_uniforms;
    let Uniforms { position, normal, .. } = vertex.uniforms;

    let view_dir = (camera - position).normalize();

    let shininess = 64.0;

    let material_color = decode_gamma(Color { r: 0.25, g: 0.25, b: 0.25, a: 1.0 }, SRGB_GAMMA);

    let albedo = 0.7;

    let mut color: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };

    for light in lights {
        let light_position = light.position.to_homogeneous();

        let light_dir = (light_position - position).normalize();
        let halfway_vector = (light_dir + view_dir).normalize();

        let NdotL = saturate(light_dir.dot(&normal));
        let NdotH = saturate(normal.dot(&halfway_vector));
        let VdotH = saturate(view_dir.dot(&halfway_vector));

        // Fresnel is used to blend together specular and diffuse lighting
        let f = fresnel_schlick(VdotH, 1.45);

        // simple Lambertian diffuse
        let diffuse = (1.0 - f) * NdotL;

        // simple Blinn-Phong specular
        let specular = f * NdotH.powf(shininess * 2.0);

        color = Color {
            r: color.r + light.intensity * light.color.r * (specular + (diffuse * albedo * material_color.r)),
            g: color.g + light.intensity * light.color.g * (specular + (diffuse * albedo * material_color.g)),
            b: color.b + light.intensity * light.color.b * (specular + (diffuse * albedo * material_color.b)),
            a: color.a,
        };
    }

    Fragment::Color(encode_gamma(aces_filmic_tonemap(color), SRGB_GAMMA))
}