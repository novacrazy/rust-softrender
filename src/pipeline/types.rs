//! Helper type aliases

use ::pixels::PixelBuffer;
use ::framebuffer::types::StencilAttachment;

use super::PipelineObject;

/// Color attachment for a given pipeline object's framebuffer
pub type Pixel<P> = <<P as PipelineObject>::Framebuffer as PixelBuffer>::Color;

/// Global uniforms for a pipeline object
pub type PipelineUniforms<P> = <P as PipelineObject>::Uniforms;

pub type StencilValue<P> = StencilAttachment<<P as PipelineObject>::Framebuffer>;
