use std::ops::{Index, IndexMut};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Indices {0} and {1} are out of bounds")]
    IndicesOutOfBounds(usize, usize),
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Grid<T, const W: usize, const H: usize, const SIZE: usize> {
    pub data: [T; SIZE],
}

impl<T: Clone + Copy + Default, const W: usize, const H: usize, const SIZE: usize> Default for Grid<T, W, H, SIZE> {
    fn default() -> Self {
        assert!(SIZE == W * H, "SIZE must be equal to W * H");
        Self {
            data: [T::default(); SIZE],
        }
    }
}

impl<T, const W: usize, const H: usize, const SIZE: usize> Grid<T, W, H, SIZE> {
    pub const fn new(data: [T; SIZE]) -> Self {
        assert!(SIZE == W * H, "SIZE must be equal to W * H");
        Self { data }
    }

    pub fn filled_with(value: T) -> Self
    where
        T: Clone,
    {
        assert!(SIZE == W * H, "SIZE must be equal to W * H");
        Self {
            data: core::array::from_fn(|_| value.clone()),
        }
    }

    pub fn idx(&self, x: usize, y: usize) -> Option<usize> {
        if x < W && y < H {
            Some(y * W + x)
        } else {
            None
        }
    }

    fn direct_idx(x: usize, y: usize) -> usize {
        y * W + x
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.data.get(y * W + x)
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.data.get_mut(y * W + x)
    }

    pub fn set(&mut self, x: usize, y: usize, value: T) {
        assert!(x < W, "x must be less than W");
        assert!(y < H, "y must be less than H");
        self.data[y * W + x] = value;
    }

    // iterators from array2d: https://github.com/HarrisonMc555/array2d

    pub fn row_iter(&self, row_index: usize) -> Result<impl DoubleEndedIterator<Item = &T> + Clone, Error> {
        let start = self.idx(0, row_index)
            .ok_or(Error::IndicesOutOfBounds(row_index, 0))?;
        let end = start + W;
        Ok(self.data[start..end].iter())
    }

    pub fn column_iter(
        &self,
        column_index: usize,
    ) -> Result<impl DoubleEndedIterator<Item = &T> + Clone, Error> {
        if column_index >= H {
            return Err(Error::IndicesOutOfBounds(0, column_index));
        }
        Ok((0..H).map(move |row_index| &self[(row_index, column_index)]))
    }

    pub fn rows_iter(
        &self,
    ) -> impl DoubleEndedIterator<Item = impl DoubleEndedIterator<Item = &T> + Clone> + Clone {
        (0..H).map(move |row_index| {
            self.row_iter(row_index)
                .expect("rows_iter should never fail")
        })
    }

    pub fn columns_iter(
        &self,
    ) -> impl DoubleEndedIterator<Item = impl DoubleEndedIterator<Item = &T> + Clone> + Clone {
        (0..W).map(move |column_index| {
            self.column_iter(column_index)
                .expect("columns_iter should never fail")
        })
    }

    pub fn indices_column_major(&self) -> impl DoubleEndedIterator<Item = (usize, usize)> + Clone {
        indices_column_major(W, H)
    }

    pub fn indices_row_major(&self) -> impl DoubleEndedIterator<Item = (usize, usize)> + Clone {
        indices_row_major(W, H)
    }
}


fn indices_row_major(
    width: usize,
    height: usize,
) -> impl DoubleEndedIterator<Item = (usize, usize)> + Clone {
    (0..height).flat_map(move |row| (0..width).map(move |column| (column, row)))
}

fn indices_column_major(
    width: usize,
    height: usize,
) -> impl DoubleEndedIterator<Item = (usize, usize)> + Clone {
    (0..width).flat_map(move |column| (0..height).map(move |row| (column, row)))
}

impl<T, const W: usize, const H: usize, const SIZE: usize> Index<(usize, usize)> for Grid<T, W, H, SIZE> {
    type Output = T;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self.data[Self::direct_idx(x, y)]
    }
}

impl<T, const W: usize, const H: usize, const SIZE: usize> IndexMut<(usize, usize)> for Grid<T, W, H, SIZE> {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        &mut self.data[Self::direct_idx(x, y)]
    }
}
