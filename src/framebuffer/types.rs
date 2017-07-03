use super::{FramebufferBase, Attachments};

pub type ColorAttachment<F> = <<F as FramebufferBase>::Attachments as Attachments>::Color;
pub type DepthAttachment<F> = <<F as FramebufferBase>::Attachments as Attachments>::Depth;
pub type StencilAttachment<F> = <<F as FramebufferBase>::Attachments as Attachments>::Stencil;