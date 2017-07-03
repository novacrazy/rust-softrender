use super::Framebuffer;
use super::types::{DepthAttachment, StencilAttachment};

pub struct FramebufferAccessor<'a, F: 'a> {
    pub ( in ::framebuffer) buffer: &'a F,
    pub ( in ::framebuffer) index: usize,
}

impl<'a, F: 'a> Clone for FramebufferAccessor<'a, F> {
    #[inline]
    fn clone(&self) -> FramebufferAccessor<'a, F> {
        FramebufferAccessor { ..*self }
    }
}

impl<'a, F: 'a> Copy for FramebufferAccessor<'a, F> {}

pub struct FramebufferAccessorMut<'a, F: 'a> {
    pub ( in ::framebuffer) buffer: &'a mut F,
    pub ( in ::framebuffer) index: usize,
}

impl<'a, F: 'a> FramebufferAccessor<'a, F> where F: Framebuffer {
    #[inline]
    pub ( in ::framebuffer) fn new(index: usize, buffer: &'a F) -> FramebufferAccessor<'a, F> {
        FramebufferAccessor { buffer, index }
    }

    #[inline]
    pub fn get_depth(&self) -> DepthAttachment<F> {
        unsafe { self.buffer.get_depth_unchecked(self.index) }
    }

    #[inline]
    pub fn get_stencil(&self) -> StencilAttachment<F> {
        unsafe { self.buffer.get_stencil_unchecked(self.index) }
    }
}

impl<'a, F: 'a> FramebufferAccessorMut<'a, F> where F: Framebuffer {
    #[inline]
    pub ( in ::framebuffer) fn new(index: usize, buffer: &'a mut F) -> FramebufferAccessorMut<'a, F> {
        FramebufferAccessorMut { buffer, index }
    }

    #[inline]
    pub fn get_depth(&self) -> DepthAttachment<F> {
        unsafe { self.buffer.get_depth_unchecked(self.index) }
    }

    #[inline]
    pub fn get_stencil(&self) -> StencilAttachment<F> {
        unsafe { self.buffer.get_stencil_unchecked(self.index) }
    }

    #[inline]
    pub fn set_depth(&mut self, depth: DepthAttachment<F>) {
        unsafe { self.buffer.set_depth_unchecked(self.index, depth) }
    }

    #[inline]
    pub fn set_stencil(&mut self, stencil: StencilAttachment<F>) {
        unsafe { self.buffer.set_stencil_unchecked(self.index, stencil) }
    }

    #[inline]
    pub fn into_ref(self) -> FramebufferAccessor<'a, F> {
        FramebufferAccessor::new(self.index, self.buffer)
    }
}