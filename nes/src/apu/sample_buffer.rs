use bincode::{Decode, Encode};

#[derive(Encode, Decode)]
pub struct SampleBuffer<T: Default + Clone + Copy + 'static, const SIZE: usize> {
    buf: Box<[T; SIZE]>,
    index: usize,
}

impl<T: Default + Clone + Copy, const SIZE: usize> SampleBuffer<T, SIZE> {
    pub fn new() -> Self {
        Self {
            buf: Box::new([T::default(); SIZE]),
            index: 0,
        }
    }

    pub fn push(&mut self, val: T) {
        self.buf[self.index] = val;
        self.index += 1;

        if self.index == SIZE {
            self.index = 0;
        }
    }

    pub fn len(&self) -> usize {
        SIZE
    }

    pub fn get_samples(&self) -> (&[T], &[T]) {
        let (s1, s2) = self.buf.as_slice().split_at(self.index);
        (s2, s1)
    }
}
