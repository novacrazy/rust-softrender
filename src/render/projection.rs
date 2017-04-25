use nalgebra::{Matrix4, Point3, Vector4};

pub fn project_point_to_screen(view: &Matrix4<f32>,
                               projection: &Matrix4<f32>,
                               viewport: (f32, f32),
                               point: &Point3<f32>) -> Vector4<f32> {
    let mut p = projection * view * point.to_homogeneous();

    p.x = (p.x / p.w + 1.0) * (viewport.0 / 2.0);
    p.y = (p.y / p.w + 1.0) * (viewport.1 / 2.0);

    p
}