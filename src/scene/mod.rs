use std::collections::LinkedList;
use std::sync::Arc;

use failure::Error;

use nalgebra::{Transform, Transform3};

use error::RenderResult;
use renderer::RenderQueue;

pub mod mesh;

pub struct Scene {
    objects: LinkedList<Box<Object>>,
}

impl Scene {
    fn enqueue_all(&self, queue: Arc<RenderQueue>) -> Result<(), Error> {
        for object in &self.objects {
            object.enqueue(queue.clone())?;
        }

        Ok(())
    }
}

pub trait Object {
    fn enqueue(&self, queue: Arc<RenderQueue>) -> Result<(), Error> {
        unimplemented!()
    }

    fn transform(&self) -> Transform3<f32> {
        Transform::identity()
    }
}
