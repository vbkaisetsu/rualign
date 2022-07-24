use alloc::vec::Vec;

pub struct Array2d<T> {
    width: usize,
    _height: usize,
    data: Vec<T>,
}

impl<T> Array2d<T>
where
    T: Default + Clone,
{
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            _height: height,
            data: vec![T::default(); width * height],
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.data.get(x + self.width * y)
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.data.get_mut(x + self.width * y)
    }

    pub fn fill(&mut self, value: T) {
        self.data.fill(value);
    }

    pub fn resize(&mut self, width: usize, height: usize, value: T) {
        self.width = width;
        self._height = height;
        self.data.resize(width * height, value);
    }
}
