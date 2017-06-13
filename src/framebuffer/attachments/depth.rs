//! Depth Buffer attachment definition

use num_traits::Bounded;

/// Defines a depth buffer attachment.
///
/// This is automatically implemented for type that satisfy the dependent traits
pub trait Depth: super::Attachment + Bounded + PartialOrd {}

impl<T> Depth for T where T: super::Attachment + Bounded + PartialOrd {}