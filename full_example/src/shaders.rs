use softrender::mesh::Vertex;
use softrender::render::{ScreenVertex, ClipVertex, Fragment, PrimitiveStorage, PrimitiveRef, Interpolate};

use ::color::{Color, SRGB_GAMMA, decode_gamma, encode_gamma, aces_filmic_tonemap};
use ::mesh::VertexData;
use ::uniforms::{GlobalUniforms, Uniforms};

pub fn vertex_shader(vertex: &Vertex<VertexData>, global_uniforms: &GlobalUniforms) -> ClipVertex<Uniforms> {
    let GlobalUniforms { ref view, ref projection, ref model, ref mit, .. } = *global_uniforms;
    let VertexData { normal, uv } = vertex.vertex_data;

    let position = vertex.position.to_homogeneous();

    // Transform vertex position to world-space
    let world_position = model * position;

    // Transform normal to world-space
    let normal = (mit * normal.to_homogeneous()).normalize();

    let mvp = projection * view * model;

    // Transform vertex position to clip-space (projection-space)
    let clip_position = mvp * position;

    // Return the clip-space position and any uniforms to interpolate and pass into the fragment shader
    ClipVertex::new(clip_position, Uniforms {
        position: world_position,
        normal: normal,
        uv: uv,
    })
}

pub const NORMAL_LENGTH: f32 = 0.05;

pub fn geometry_shader_visualize_vertex_normals<'p, 's>(mut storage: PrimitiveStorage<'p, Uniforms>,
                                                        primitive: PrimitiveRef<'s, Uniforms>,
                                                        global_uniforms: &GlobalUniforms) {
    match primitive {
        PrimitiveRef::Triangle { a, b, c } => {
            let GlobalUniforms { ref model, ref view, ref projection, .. } = *global_uniforms;

            let mv = projection * view;

            for v in &[a, b, c] {
                let Uniforms { ref position, ref normal, .. } = v.uniforms;

                let start = mv * position;
                let end = mv * (position + normal * NORMAL_LENGTH);

                storage.emit_line(ClipVertex {
                    position: start,
                    uniforms: v.uniforms.clone(),
                }, ClipVertex {
                    position: end,
                    uniforms: v.uniforms.clone(),
                })
            }
        }
        _ => storage.re_emit(primitive)
    }
}

pub fn geometry_shader_visualize_face_normals<'p, 's>(mut storage: PrimitiveStorage<'p, Uniforms>,
                                                      primitive: PrimitiveRef<'s, Uniforms>,
                                                      global_uniforms: &GlobalUniforms) {
    match primitive {
        PrimitiveRef::Triangle { a, b, c } => {
            let GlobalUniforms { ref model, ref view, ref projection, .. } = *global_uniforms;

            let mv = projection * view;

            const ONE_THIRD: f32 = 1.0 / 3.0;

            let center = Interpolate::barycentric_interpolate(ONE_THIRD, &a.uniforms, ONE_THIRD, &b.uniforms, ONE_THIRD, &c.uniforms);

            let start = mv * center.position;
            let end = mv * (center.position + center.normal.normalize() * NORMAL_LENGTH);

            storage.emit_line(ClipVertex {
                position: start,
                uniforms: center.clone(),
            }, ClipVertex {
                position: end,
                uniforms: center.clone(),
            });
        }
        _ => storage.re_emit(primitive)
    }
}

// Simple Fresnel Schlick approximation
fn fresnel_schlick(cos_theta: f32, ior: f32) -> f32 {
    let f0 = ((1.0 - ior) / (1.0 + ior)).powi(2);

    f0 + (1.0 - f0) * (1.0 - cos_theta).powi(5)
}

fn saturate(value: f32) -> f32 {
    if value < 0.0 { 0.0 } else if value > 1.0 { 1.0 } else { value }
}

pub fn fragment_shader_green(_: &ScreenVertex<Uniforms>, _: &GlobalUniforms) -> Fragment<Color> {
    Fragment::Color(Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 })
}

// GLSL habits die hard
#[allow(non_snake_case)]
pub fn fragment_shader(vertex: &ScreenVertex<Uniforms>, global_uniforms: &GlobalUniforms) -> Fragment<Color> {
    //return Fragment::Color(Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 });

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