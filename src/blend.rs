//! Pixel blending

use ::pixel::Pixel;

use std::sync::Arc;
use std::marker::PhantomData;

pub trait Blend<P: Pixel>: Send + Sync {
    /// The first parameter passed to the blend function is the output of the fragment shader, the source color.
    ///
    /// The second parameter passed to the blend function is the existing value in the framebuffer to blend over.
    ///
    /// You can use the tool [Here](http://www.andersriggelsen.dk/glblendfunc.php) to see how OpenGL does blending,
    /// and choose how you want to blend pixels.
    ///
    /// For a generic alpha-over blend function, check the Wikipedia article [Here](https://en.wikipedia.org/wiki/Alpha_compositing)
    /// for the *over* color function.
    fn blend(&self, a: P, b: P) -> P;
}

impl<P: Pixel> Blend<P> for () {
    #[inline(always)]
    fn blend(&self, a: P, _: P) -> P { a }
}

/// Generic blend structure that can accept a user-defined blend function
#[derive(Clone)]
pub struct BoxedGenericBlend<P: Pixel> {
    blend_func: Arc<Box<Fn(P, P) -> P + Send + Sync>>,
}

impl<P: Pixel> BoxedGenericBlend<P> {
    pub fn set_blend_function<F>(&mut self, f: F) where F: Fn(P, P) -> P + Send + Sync + 'static {
        self.blend_func = Arc::new(Box::new(f))
    }
}

impl<P: Pixel> Default for BoxedGenericBlend<P> {
    fn default() -> BoxedGenericBlend<P> {
        BoxedGenericBlend {
            blend_func: Arc::new(Box::new(|a, _| a)),
        }
    }
}

impl<P: Pixel> Blend<P> for BoxedGenericBlend<P> {
    fn blend(&self, a: P, b: P) -> P {
        (**self.blend_func)(a, b)
    }
}

pub struct GenericBlend<P: Pixel, F> {
    blend_func: F,
    pixel: PhantomData<P>,
}

impl<P: Pixel, F> GenericBlend<P, F> {
    pub fn new(blend_func: F) -> GenericBlend<P, F> where F: Fn(P, P) -> P + Send + Sync + 'static  {
        GenericBlend { blend_func, pixel: PhantomData }
    }
}

impl<P: Pixel, F> Blend<P> for GenericBlend<P, F> where F: Fn(P, P) -> P + Send + Sync + 'static {
    fn blend(&self, a: P, b: P) -> P {
        (self.blend_func)(a, b)
    }
}