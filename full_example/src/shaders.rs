use softrender::mesh::Vertex;
use softrender::render::{ScreenVertex, ClipVertex, Fragment};

use ::color::{Color, SRGB_GAMMA, decode_gamma, encode_gamma, aces_filmic_tonemap};
use ::mesh::VertexData;
use ::uniforms::{GlobalUniforms, Uniforms};

pub fn vertex_shader(vertex: &Vertex<VertexData>, global_uniforms: &GlobalUniforms) -> ClipVertex<Uniforms> {
    let GlobalUniforms { ref view, ref projection, ref model, ref model_inverse_transpose, .. } = *global_uniforms;
    let VertexData { normal, uv } = vertex.vertex_data;

    // Transform vertex position to world-space
    let world_position = model * vertex.position.to_homogeneous();

    // Transform normal to world-space
    let normal = (model_inverse_transpose * normal.to_homogeneous()).normalize();

    // Transform vertex position to clip-space (projection-space)
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

    // Specular "shininess", almost like a roughness parameter but not really.
    let shininess = 32;

    // Dark grey surface
    let material_color = decode_gamma(Color { r: 0.25, g: 0.25, b: 0.25, a: 1.0 }, SRGB_GAMMA);

    // Surface albedo, or how bright diffuse lighting is
    let albedo = 0.7;

    // Start with a black fragment to accumulate lighting
    let mut color: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };

    for light in lights {
        let light_position = light.position.to_homogeneous();

        // Get distance to light for simple light attenuation
        let light_distance = (light_position - position).norm();

        let light_dir = (light_position - position).normalize();
        let halfway_vector = (light_dir + view_dir).normalize();

        // Calculate light attenuation based on inverse distance squared
        let intensity = light.intensity / light_distance.powi(2);

        let NdotL = saturate(light_dir.dot(&normal));
        let NdotH = saturate(normal.dot(&halfway_vector));
        let VdotH = saturate(view_dir.dot(&halfway_vector));

        // Fresnel is used to blend together specular and diffuse lighting
        let f = fresnel_schlick(VdotH, 1.45);

        // simple Lambertian diffuse
        let diffuse = (1.0 - f) * NdotL;

        // simple Blinn-Phong specular
        let specular = f * NdotH.powi(shininess * 2);

        // Add specular and diffuse colors together, multiple by light color,
        // and multiple by light intensity accounting for distance attenuation,
        // then add it to the previous color
        color.r += intensity * light.color.r * (specular + (diffuse * albedo * material_color.r));
        color.g += intensity * light.color.g * (specular + (diffuse * albedo * material_color.g));
        color.b += intensity * light.color.b * (specular + (diffuse * albedo * material_color.b));
    }

    // Tonemap and encode gamma for fragment
    Fragment::Color(encode_gamma(aces_filmic_tonemap(color), SRGB_GAMMA))
}