use nalgebra::Point3;

pub trait Vertex: Send + Sync {
    fn position(&self) -> &Point3<f32>;
}

pub struct GenericVertex<D> {
    pub position: Point3<f32>,
    pub data: D,
}
