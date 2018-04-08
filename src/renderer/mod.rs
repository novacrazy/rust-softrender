use std::sync::{Arc, RwLock};

use failure::Error;

pub trait Renderer {
    fn process_queue(&mut self, queue: Arc<RenderQueue>) -> Result<(), Error>;
}

pub struct RenderQueue {
    queue: RwLock<Vec<()>>
}

impl RenderQueue {
    pub fn new() -> Arc<RenderQueue> {
        Arc::new(RenderQueue {
            queue: RwLock::default()
        })
    }
}