//! `RenderBuffer` iterator support

use std::slice;

use ::attachments::{Attachments, Stencil};

/// Contains a reference to a `RenderBuffer` pixel value
pub struct RenderBufferPixelRef<'a, A: Attachments> {
    item: &'a (A::Color, A::Depth, <A::Stencil as Stencil>::Type),
}

impl<'a, A: Attachments> RenderBufferPixelRef<'a, A> {
    /// Return a reference to the pixel color value
    pub fn color(&self) -> &A::Color { &self.item.0 }
    /// Return a reference to the pixel depth value
    pub fn depth(&self) -> &A::Depth { &self.item.1 }
    /// Return a reference to the pixel stencil value
    pub fn stencil(&self) -> &<A::Stencil as Stencil>::Type { &self.item.2 }
}

/// Contains a mutable reference to a `RenderBuffer` pixel value
pub struct RenderBufferPixelMut<'a, A: Attachments> {
    item: &'a mut (A::Color, A::Depth, <A::Stencil as Stencil>::Type),
}

impl<'a, A: Attachments> RenderBufferPixelMut<'a, A> {
    /// Return a reference to the pixel color value
    pub fn color(&self) -> &A::Color { &self.item.0 }
    /// Return a reference to the pixel depth value
    pub fn depth(&self) -> &A::Depth { &self.item.1 }
    /// Return a reference to the pixel stencil value
    pub fn stencil(&self) -> &<A::Stencil as Stencil>::Type { &self.item.2 }

    /// Return a mutable reference to the pixel color value
    pub fn color_mut(&mut self) -> &mut A::Color { &mut self.item.0 }
    /// Return a mutable reference to the pixel depth value
    pub fn depth_mut(&mut self) -> &mut A::Depth { &mut self.item.1 }
    /// Return a mutable reference to the pixel stencil value
    pub fn stencil_mut(&mut self) -> &mut <A::Stencil as Stencil>::Type { &mut self.item.2 }
}

/// Iterator for `RenderBuffer` pixel values
pub struct RenderBufferIter<'a, A: Attachments> {
    pub ( in ::framebuffer::renderbuffer) iter: slice::Iter<'a, (A::Color, A::Depth, <A::Stencil as Stencil>::Type)>,
}

/// Iterator for mutating `RenderBuffer` pixel values
pub struct RenderBufferIterMut<'a, A: Attachments> {
    pub ( in ::framebuffer::renderbuffer) iter: slice::IterMut<'a, (A::Color, A::Depth, <A::Stencil as Stencil>::Type)>,
}

impl<'a, A: Attachments> Iterator for RenderBufferIter<'a, A> {
    type Item = RenderBufferPixelRef<'a, A>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|item| RenderBufferPixelRef { item })
    }
}

impl<'a, A: Attachments> DoubleEndedIterator for RenderBufferIter<'a, A> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|item| RenderBufferPixelRef { item })
    }
}

impl<'a, A: Attachments> Iterator for RenderBufferIterMut<'a, A> {
    type Item = &'a mut A::Color;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|&mut (ref mut color, _, _)| color)
    }
}

impl<'a, A: Attachments> DoubleEndedIterator for RenderBufferIterMut<'a, A> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|&mut (ref mut color, _, _)| color)
    }
}