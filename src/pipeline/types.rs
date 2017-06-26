//! Helper type aliases

use ::framebuffer::{Framebuffer, Attachments};
use super::PipelineObject;

/// Color attachment for a given pipeline object's framebuffer
pub type ColorAttachment<P> = <<<P as PipelineObject>::Framebuffer as Framebuffer>::Attachments as Attachments>::Color;

/// Global uniforms for a pipeline object
pub type PipelineUniforms<P> = <P as PipelineObject>::Uniforms;
