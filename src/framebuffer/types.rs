use super::{Framebuffer, Attachments};

pub type ColorAttachment<F> = <<F as Framebuffer>::Attachments as Attachments>::Color;
pub type DepthAttachment<F> = <<F as Framebuffer>::Attachments as Attachments>::Depth;
pub type StencilAttachment<F> = <<F as Framebuffer>::Attachments as Attachments>::Stencil;
