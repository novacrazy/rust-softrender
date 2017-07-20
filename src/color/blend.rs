//! Defines color blending trait and standard blend function handling

use std::sync::Arc;
use std::marker::PhantomData;

use super::Color;

/// Defines some kind of color blending function
pub trait Blend<C: Color>: Send + Sync {
    /// The first parameter passed to the blend function is the output of the fragment shader, the source color.
    ///
    /// The second parameter passed to the blend function is the existing value in the framebuffer to blend over.
    ///
    /// You can use the tool [Here](http://www.andersriggelsen.dk/glblendfunc.php) to see how OpenGL does blending,
    /// and choose how you want to blend colors.
    ///
    /// For a generic alpha-over blend function, check the Wikipedia article [Here](https://en.wikipedia.org/wiki/Alpha_compositing)
    /// for the *over* color function.
    fn blend(&self, a: C, b: C) -> C;
}

impl<'a, B, C: Color> Blend<C> for &'a B where B: Blend<C> {
    fn blend(&self, a: C, b: C) -> C {
        (**self).blend(a, b)
    }
}

impl<C: Color> Blend<C> for () {
    #[inline(always)]
    fn blend(&self, a: C, _: C) -> C { a }
}

/// Generic blend structure that can accept a user-defined blend function at runtime
#[derive(Clone)]
pub struct BoxedGenericBlend<C: Color> {
    blend_func: Arc<Box<Fn(C, C) -> C + Send + Sync>>,
}

impl<C: Color> BoxedGenericBlend<C> {
    pub fn set_blend_function<F>(&mut self, f: F) where F: Fn(C, C) -> C + Send + Sync + 'static {
        self.blend_func = Arc::new(Box::new(f))
    }
}

impl<C: Color> Default for BoxedGenericBlend<C> {
    fn default() -> BoxedGenericBlend<C> {
        BoxedGenericBlend {
            blend_func: Arc::new(Box::new(|a, _| a)),
        }
    }
}

impl<C: Color> Blend<C> for BoxedGenericBlend<C> {
    fn blend(&self, a: C, b: C) -> C {
        (**self.blend_func)(a, b)
    }
}

/// Generic blend structure that can accept a user-defined blend function at compile time
pub struct GenericBlend<C: Color, F> {
    blend_func: F,
    color: PhantomData<C>,
}

impl<C: Color, F> GenericBlend<C, F> {
    pub fn new(blend_func: F) -> GenericBlend<C, F> where F: Fn(C, C) -> C + Send + Sync + 'static {
        GenericBlend { blend_func, color: PhantomData }
    }
}

impl<C: Color, F> Blend<C> for GenericBlend<C, F> where F: Fn(C, C) -> C + Send + Sync + 'static {
    fn blend(&self, a: C, b: C) -> C {
        (self.blend_func)(a, b)
    }
}