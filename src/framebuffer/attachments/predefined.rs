//! Predefined attachment structures and aliases

use ::framebuffer::attachments::{Attachments, Color, Depth, Stencil};

use std::marker::PhantomData;

/// Marker structure for specifying common Color/Depth/Stencil attachments for framebuffers.
///
/// This type cannot be instantiated and is zero-sized, only used as a marker.
#[derive(Debug, Clone, Copy, Default)]
pub struct ColorDepthStencilAttachments<C: Color, D: Depth, S: Stencil>(PhantomData<(C, D, S)>);

impl<C: Color, D: Depth, S: Stencil> Attachments for ColorDepthStencilAttachments<C, D, S> {
    type Color = C;
    type Depth = D;
    type Stencil = S;
}

pub type ColorAttachment<C> = ColorDepthStencilAttachments<C, (), ()>;
pub type ColorDepthAttachments<C, D> = ColorDepthStencilAttachments<C, D, ()>;
pub type DepthAttachment<D> = ColorDepthStencilAttachments<(), D, ()>;
pub type DepthStencilAttachments<D, S> = ColorDepthStencilAttachments<(), D, S>;
pub type StencilAttachment<S> = ColorDepthStencilAttachments<(), (), S>;
pub type ColorStencilAttachments<C, S> = ColorDepthStencilAttachments<C, (), S>;

pub type EmptyAttachments = ColorDepthStencilAttachments<(), (), ()>;

#[cfg(test)]
mod test {
    use std::mem::size_of;
    use ::framebuffer::attachments::{ColorDepthStencilAttachments, GenericStencil};

    #[test]
    fn test_cds_attachments_size() {
        assert_eq!(0, size_of::<ColorDepthStencilAttachments<(), f32, GenericStencil<u8>>>())
    }
}