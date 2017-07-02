use super::{Framebuffer, Attachments};
use super::attachments::Stencil;

pub type ColorAttachment<F> = <<F as Framebuffer>::Attachments as Attachments>::Color;
pub type DepthAttachment<F> = <<F as Framebuffer>::Attachments as Attachments>::Depth;
pub type StencilAttachment<F> = <<<F as Framebuffer>::Attachments as Attachments>::Stencil as Stencil>::Type;
pub type StencilConfig<F> = <<<F as Framebuffer>::Attachments as Attachments>::Stencil as Stencil>::Config;