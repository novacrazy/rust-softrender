//! Type Behavior Traits

pub trait ThreadSafeCopyable: Copy + Send + Sync + 'static {}

impl<T> ThreadSafeCopyable for T where T: Copy + Send + Sync + 'static {}