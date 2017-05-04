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
    fn blend(&self, P, P) -> P;

    #[inline]
    fn blend_by_depth(&self, foreground_depth: f32, background_depth: f32, a: P, b: P) -> P {
        if background_depth > foreground_depth {
            self.blend(a, b)
        } else {
            self.blend(b, a)
        }
    }

    fn ignore_depth(&self) -> bool;
}

impl<P: Pixel> Blend<P> for () {
    #[inline(always)]
    fn blend(&self, a: P, _: P) -> P { a }

    #[inline(always)]
    fn ignore_depth(&self) -> bool { false }
}

/// Generic blend structure that can accept a user-defined blend function
#[derive(Clone)]
pub struct GenericBlend<P: Pixel> {
    blend_func: Arc<Box<Fn(P, P) -> P + Send + Sync>>,
    ignore_depth: bool,
}

impl<P: Pixel> GenericBlend<P> {
    pub fn set_blend_function<F>(&mut self, f: F) where F: Fn(P, P) -> P + Send + Sync + 'static {
        self.blend_func = Arc::new(Box::new(f))
    }

    pub fn ignore_depth(&mut self, enable: bool) {
        self.ignore_depth = enable;
    }
}

impl<P: Pixel> Default for GenericBlend<P> {
    fn default() -> GenericBlend<P> {
        GenericBlend {
            blend_func: Arc::new(Box::new(|a, _| a)),
            ignore_depth: false
        }
    }
}

impl<P: Pixel> Blend<P> for GenericBlend<P> {
    #[inline(always)]
    fn blend(&self, a: P, b: P) -> P {
        (**self.blend_func)(a, b)
    }

    #[inline(always)]
    fn ignore_depth(&self) -> bool { self.ignore_depth }
}

/*
#[derive(Debug)]
pub struct BlendOver<P: Pixel>(PhantomData<P>);

impl<P: Pixel> Clone for BlendOver<P> {
    #[inline(always)]
    fn clone(&self) -> BlendOver<P> { BlendOver(PhantomData) }
}

impl<P: Pixel> Copy for BlendOver<P> {}

impl<P: Pixel> Blend<P> for BlendOver<P> {
    fn blend(&self, a: P, b: P) -> P {
        //TODO
    }
}
*/