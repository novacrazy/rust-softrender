//! Helper type aliases

use ::pixels::PixelBuffer;
use ::framebuffer::Framebuffer;

use super::PipelineObject;

/// Color attachment for a given pipeline object's framebuffer
pub type Pixel<P> = <<P as PipelineObject>::Framebuffer as PixelBuffer>::Color;

/// Global uniforms for a pipeline object
pub type PipelineUniforms<P> = <P as PipelineObject>::Uniforms;
