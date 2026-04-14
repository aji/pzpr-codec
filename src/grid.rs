use std::ops::{Index, IndexMut};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Shape(pub isize, pub isize);

impl Shape {
    pub fn rows(&self) -> isize {
        self.0
    }
    pub fn cols(&self) -> isize {
        self.1
    }
    pub fn len(&self) -> isize {
        self.rows() * self.cols()
    }

    pub fn is_valid_rc(&self, r: isize, c: isize) -> bool {
        0 <= r && r < self.rows() && 0 <= c && c < self.cols()
    }
    pub fn is_valid_idx(&self, idx: isize) -> bool {
        0 <= idx && idx < self.len()
    }

    pub fn rc_to_idx(&self, r: isize, c: isize) -> isize {
        r * self.cols() + c
    }
    pub fn idx_to_rc(&self, idx: isize) -> (isize, isize) {
        (idx / self.cols(), idx % self.cols())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Stride(pub isize, pub isize);

impl Stride {
    pub fn per_row(&self) -> isize {
        self.0
    }
    pub fn per_col(&self) -> isize {
        self.1
    }

    pub fn rc_to_offset(&self, r: isize, c: isize) -> usize {
        let x = r * self.per_row() + c * self.per_col();
        if x < 0 {
            panic!("out of bounds: {r},{c}");
        }
        x as usize
    }
}

pub trait Gridlike<T> {
    fn shape(&self) -> Shape;

    fn stride(&self) -> Stride;

    fn buffer(&self) -> &[T];

    fn len(&self) -> usize {
        self.shape().len() as usize
    }

    fn rc(&self, r: isize, c: isize) -> &T {
        let shape = self.shape();
        let stride = self.stride();
        let buf = self.buffer();
        if !shape.is_valid_rc(r, c) {
            panic!("invalid rc: {r},{c}");
        }
        let offset = stride.rc_to_offset(r, c);
        match offset < buf.len() {
            true => &buf[offset as usize],
            false => panic!("internal error: {r},{c} -> {offset}"),
        }
    }

    fn idx(&self, idx: isize) -> &T {
        let (r, c) = self.shape().idx_to_rc(idx);
        self.rc(r, c)
    }

    fn view<'g>(&'g self, row0: isize, col0: isize, rows: isize, cols: isize) -> GridView<'g, T> {
        let shape = self.shape();
        let stride = self.stride();
        let buf = self.buffer();
        if rows < 1 || cols < 1 {
            panic!("invalid grid view size: {rows},{cols}");
        }
        let row1 = row0 + rows - 1;
        let col1 = col0 + cols - 1;
        if !shape.is_valid_rc(row0, col0) {
            panic!("invalid grid view start: {row0},{col0}");
        }
        if !shape.is_valid_rc(row1, col1) {
            panic!("invalid grid view: {row0},{col0}+{rows},{cols}");
        }
        let offset = stride.rc_to_offset(row0, col0);
        match offset < buf.len() {
            true => GridView {
                shape: Shape(rows, cols),
                stride,
                buffer: &buf[offset as usize..],
            },
            false => panic!("internal error: {row0},{col0} -> {offset}"),
        }
    }

    fn cells<'g>(&'g self) -> Cells<'g, T> {
        Cells {
            next: 0,
            shape: self.shape(),
            stride: self.stride(),
            buffer: self.buffer(),
        }
    }
}

pub trait GridlikeMut<T>: Gridlike<T> {
    fn buffer_mut(&mut self) -> &mut [T];

    fn rc_mut(&mut self, r: isize, c: isize) -> &mut T {
        let shape = self.shape();
        let stride = self.stride();
        let buf = self.buffer_mut();
        if !shape.is_valid_rc(r, c) {
            panic!("invalid rc: {r},{c}");
        }
        let offset = stride.rc_to_offset(r, c);
        match offset < buf.len() {
            true => &mut buf[offset as usize],
            false => panic!("internal error: {r},{c} -> {offset}"),
        }
    }

    fn idx_mut(&mut self, idx: isize) -> &mut T {
        let (r, c) = self.shape().idx_to_rc(idx);
        self.rc_mut(r, c)
    }

    fn view_mut<'g>(
        &'g mut self,
        row0: isize,
        col0: isize,
        rows: isize,
        cols: isize,
    ) -> GridViewMut<'g, T> {
        let shape = self.shape();
        let stride = self.stride();
        let buf = self.buffer_mut();
        if rows < 1 || cols < 1 {
            panic!("invalid grid view size: {rows},{cols}");
        }
        let row1 = row0 + rows - 1;
        let col1 = col0 + cols - 1;
        if !shape.is_valid_rc(row0, col0) {
            panic!("invalid grid view start: {row0},{col0}");
        }
        if !shape.is_valid_rc(row1, col1) {
            panic!("invalid grid view: {row0},{col0}+{rows},{cols}");
        }
        let offset = stride.rc_to_offset(row0, col0);
        match offset < buf.len() {
            true => GridViewMut {
                shape: Shape(rows, cols),
                stride,
                buffer: &mut buf[offset as usize..],
            },
            false => panic!("internal error: {row0},{col0} -> {offset}"),
        }
    }
}

pub struct Cells<'g, T> {
    next: isize,
    shape: Shape,
    stride: Stride,
    buffer: &'g [T],
}

impl<'g, T> Iterator for Cells<'g, T> {
    type Item = &'g T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next >= self.shape.len() {
            None
        } else {
            let (r, c) = self.shape.idx_to_rc(self.next);
            let res = &self.buffer[self.stride.rc_to_offset(r, c)];
            self.next += 1;
            Some(res)
        }
    }
}

#[derive(Clone)]
pub struct Grid<T> {
    shape: Shape,
    buffer: Vec<T>,
}

impl<T> Grid<T> {
    pub fn new_filled(rows: isize, cols: isize, value: T) -> Grid<T>
    where
        T: Clone,
    {
        if rows < 1 || cols < 1 {
            panic!("invalid grid size: {rows},{cols}");
        }
        Grid {
            shape: Shape(rows, cols),
            buffer: (0..rows * cols).map(|_| value.clone()).collect(),
        }
    }

    pub fn new_fill_with<F>(rows: isize, cols: isize, f: F) -> Grid<T>
    where
        F: Fn(isize, isize) -> T,
    {
        if rows < 1 || cols < 1 {
            panic!("invalid grid size: {rows},{cols}");
        }
        Grid {
            shape: Shape(rows as isize, cols as isize),
            buffer: (0..rows * cols).map(|i| f(i / cols, i % cols)).collect(),
        }
    }

    pub fn reshape(self, rows: isize, cols: isize) -> Result<Self, String> {
        if rows < 1 || cols < 1 {
            return Err(format!("invalid grid size: {rows},{cols}"));
        }
        if rows * cols != self.shape().len() {
            return Err(format!(
                "invalid reshape: {},{} -> {rows},{cols}",
                self.shape().rows(),
                self.shape().cols()
            ));
        }
        Ok(Grid {
            shape: Shape(rows, cols),
            buffer: self.buffer,
        })
    }
}

impl<T> Gridlike<T> for Grid<T> {
    fn shape(&self) -> Shape {
        self.shape
    }

    fn stride(&self) -> Stride {
        Stride(self.shape.cols(), 1)
    }

    fn buffer(&self) -> &[T] {
        &self.buffer[..]
    }
}

impl<T> GridlikeMut<T> for Grid<T> {
    fn buffer_mut(&mut self) -> &mut [T] {
        &mut self.buffer[..]
    }
}

impl<T> FromIterator<T> for Grid<T> {
    fn from_iter<It: IntoIterator<Item = T>>(iter: It) -> Self {
        let buffer: Vec<T> = iter.into_iter().collect();
        Grid {
            shape: Shape(1, buffer.len() as isize),
            buffer,
        }
    }
}

impl<T> Index<usize> for Grid<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.idx(index as isize)
    }
}

impl<T> IndexMut<usize> for Grid<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.idx_mut(index as isize)
    }
}

#[derive(Copy, Clone)]
pub struct GridView<'g, T> {
    shape: Shape,
    stride: Stride,
    buffer: &'g [T],
}

impl<'g, T> Gridlike<T> for GridView<'g, T> {
    fn shape(&self) -> Shape {
        self.shape
    }

    fn stride(&self) -> Stride {
        self.stride
    }

    fn buffer(&self) -> &[T] {
        self.buffer
    }
}

impl<'g, T> Index<usize> for GridView<'g, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.idx(index as isize)
    }
}

pub struct GridViewMut<'g, T> {
    shape: Shape,
    stride: Stride,
    buffer: &'g mut [T],
}

impl<'g, T> Gridlike<T> for GridViewMut<'g, T> {
    fn shape(&self) -> Shape {
        self.shape
    }

    fn stride(&self) -> Stride {
        self.stride
    }

    fn buffer(&self) -> &[T] {
        self.buffer
    }
}

impl<'g, T> GridlikeMut<T> for GridViewMut<'g, T> {
    fn buffer_mut(&mut self) -> &mut [T] {
        self.buffer
    }
}

impl<'g, T> Index<usize> for GridViewMut<'g, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.idx(index as isize)
    }
}

impl<'g, T> IndexMut<usize> for GridViewMut<'g, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.idx_mut(index as isize)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_grid_view() {
        let g: Grid<isize> = Grid::new_fill_with(5, 5, |r, c| r + 2 * c);
        assert_eq!(g.shape(), Shape(5, 5));
        assert_eq!(g.stride(), Stride(5, 1));
        assert!(g.shape().is_valid_rc(4, 4));
        assert!(!g.shape().is_valid_rc(0, -1));
        assert!(!g.shape().is_valid_rc(-1, 0));
        assert!(!g.shape().is_valid_rc(4, 5));
        assert!(!g.shape().is_valid_rc(5, 4));
        assert_eq!(*g.rc(0, 0), 0);
        assert_eq!(*g.rc(2, 1), 4);
        assert_eq!(*g.rc(4, 4), 12);
        assert_eq!(g.len(), 25);

        let v0 = g.view(0, 0, 3, 3);
        assert_eq!(v0.shape(), Shape(3, 3));
        assert_eq!(v0.stride(), Stride(5, 1));
        assert!(v0.shape().is_valid_rc(2, 2));
        assert!(!v0.shape().is_valid_rc(0, -1));
        assert!(!v0.shape().is_valid_rc(-1, 0));
        assert!(!v0.shape().is_valid_rc(2, 3));
        assert!(!v0.shape().is_valid_rc(3, 2));
        assert_eq!(*v0.rc(0, 0), 0);
        assert_eq!(*v0.rc(2, 1), 4);
        assert_eq!(v0.len(), 9);

        let v1 = v0.view(1, 1, 2, 2);
        assert_eq!(v1.shape(), Shape(2, 2));
        assert_eq!(v1.stride(), Stride(5, 1));
        assert!(v1.shape().is_valid_rc(1, 1));
        assert!(!v1.shape().is_valid_rc(0, -1));
        assert!(!v1.shape().is_valid_rc(-1, 0));
        assert!(!v1.shape().is_valid_rc(1, 2));
        assert!(!v1.shape().is_valid_rc(2, 1));
        assert_eq!(*v1.rc(0, 0), 3);
        assert_eq!(*v1.rc(1, 1), 6);
        assert_eq!(v1.len(), 4);
        assert_eq!(v1[0], 3);
        assert_eq!(v1[1], 5);
        assert_eq!(v1[2], 4);
        assert_eq!(v1[3], 6);
    }

    #[test]
    fn test_grid_mut() {
        let mut g: Grid<isize> = Grid::new_filled(5, 5, 0);

        assert_eq!(g.len(), 25);
        assert_eq!(g[0], 0);
        assert_eq!(g[5], 0);

        g[5] = 3;
        assert_eq!(g[5], 3);
        assert_eq!(*g.rc(1, 0), 3);

        let mut v = g.view_mut(1, 1, 2, 2);
        assert_eq!(v.len(), 4);
        v[0] = 10;
        v[1] = 11;
        v[2] = 12;
        v[3] = 13;

        assert_eq!(*g.rc(1, 1), 10);
        assert_eq!(*g.rc(1, 2), 11);
        assert_eq!(*g.rc(2, 1), 12);
        assert_eq!(*g.rc(2, 2), 13);
    }
}
