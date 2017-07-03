//! `RenderBuffer` iterator support

use std::slice;

use ::attachments::{Attachments, Stencil};

use super::RenderBufferAttachments;

/// Contains a reference to a `RenderBuffer` pixel value
pub struct RenderBufferPixelRef<'a, A: Attachments> {
    pixel: &'a RenderBufferAttachments<A>,
}

impl<'a, A: Attachments> Clone for RenderBufferPixelRef<'a, A> {
    fn clone(&self) -> RenderBufferPixelRef<'a, A> {
        RenderBufferPixelRef { ..*self }
    }
}

impl<'a, A: Attachments> Copy for RenderBufferPixelRef<'a, A> {}

impl<'a, A: Attachments> RenderBufferPixelRef<'a, A> {
    /// Return a reference to the pixel color value
    #[inline]
    pub fn color(&self) -> &A::Color { &self.pixel.color }
    /// Return a reference to the pixel depth value
    #[inline]
    pub fn depth(&self) -> &A::Depth { &self.pixel.depth }
    /// Return a reference to the pixel stencil value
    #[inline]
    pub fn stencil(&self) -> &A::Stencil { &self.pixel.stencil }
}

/// Contains a mutable reference to a `RenderBuffer` pixel value
pub struct RenderBufferPixelMut<'a, A: Attachments> {
    item: &'a mut RenderBufferAttachments<A>,
}

impl<'a, A: Attachments> RenderBufferPixelMut<'a, A> {
    /// Return a reference to the pixel color value
    #[inline]
    pub fn color(&self) -> &A::Color { &self.item.color }
    /// Return a reference to the pixel depth value
    #[inline]
    pub fn depth(&self) -> &A::Depth { &self.item.depth }
    /// Return a reference to the pixel stencil value
    #[inline]
    pub fn stencil(&self) -> &A::Stencil { &self.item.stencil }

    /// Return a mutable reference to the pixel color value
    #[inline]
    pub fn color_mut(&mut self) -> &mut A::Color { &mut self.item.color }
    /// Return a mutable reference to the pixel depth value
    #[inline]
    pub fn depth_mut(&mut self) -> &mut A::Depth { &mut self.item.depth }
    /// Return a mutable reference to the pixel stencil value
    #[inline]
    pub fn stencil_mut(&mut self) -> &mut A::Stencil { &mut self.item.stencil }
}

/// Iterator for `RenderBuffer` pixel values
pub struct RenderBufferIter<'a, A: Attachments> {
    pub ( in ::framebuffer::renderbuffer) iter: slice::Iter<'a, RenderBufferAttachments<A>>,
}

impl<'a, A: Attachments> Clone for RenderBufferIter<'a, A> {
    fn clone(&self) -> RenderBufferIter<'a, A> {
        RenderBufferIter { iter: self.iter.clone() }
    }
}

/// Iterator for mutating `RenderBuffer` pixel values
pub struct RenderBufferIterMut<'a, A: Attachments> {
    pub ( in ::framebuffer::renderbuffer) iter: slice::IterMut<'a, RenderBufferAttachments<A>>,
}

impl<'a, A: Attachments> Iterator for RenderBufferIter<'a, A> {
    type Item = RenderBufferPixelRef<'a, A>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|item| RenderBufferPixelRef { pixel: item })
    }
}

impl<'a, A: Attachments> DoubleEndedIterator for RenderBufferIter<'a, A> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|item| RenderBufferPixelRef { pixel: item })
    }
}

impl<'a, A: Attachments> Iterator for RenderBufferIterMut<'a, A> {
    type Item = RenderBufferPixelMut<'a, A>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|item| RenderBufferPixelMut { item })
    }
}

impl<'a, A: Attachments> DoubleEndedIterator for RenderBufferIterMut<'a, A> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|item| RenderBufferPixelMut { item })
    }
}