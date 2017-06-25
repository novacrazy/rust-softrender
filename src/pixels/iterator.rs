use super::{PixelBuffer, PixelRead, PixelWrite, PixelRef, PixelMut};

pub struct PixelBufferIter<'a, P: 'a> where P: PixelRead {
    pub ( in ::pixels) buffer: &'a P,
    pub ( in ::pixels) position: usize,
    pub ( in ::pixels) max_len: usize,
}

impl<'a, P: 'a> Clone for PixelBufferIter<'a, P> where P: PixelRead {
    fn clone(&self) -> PixelBufferIter<'a, P> {
        PixelBufferIter { ..*self }
    }
}

impl<'a, P: 'a> Copy for PixelBufferIter<'a, P> where P: PixelRead {}

impl<'a, P: 'a> DoubleEndedIterator for PixelBufferIter<'a, P> where P: PixelRead {
    fn next_back(&mut self) -> Option<PixelRef<'a, P>> {
        if self.position == 0 { None } else {
            let res = PixelRef(self.position, self.buffer);
            self.position -= 1;
            Some(res)
        }
    }
}

impl<'a, P: 'a> Iterator for PixelBufferIter<'a, P> where P: PixelRead {
    type Item = PixelRef<'a, P>;

    fn next(&mut self) -> Option<PixelRef<'a, P>> {
        if self.position >= self.max_len { None } else {
            let res = PixelRef(self.position, self.buffer);
            self.position += 1;
            Some(res)
        }
    }
}