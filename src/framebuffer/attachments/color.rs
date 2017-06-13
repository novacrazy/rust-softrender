//! Color Buffer attachment definitions

/// Defines a Color buffer attachment
pub trait Color: super::Attachment {}

impl Color for () {}