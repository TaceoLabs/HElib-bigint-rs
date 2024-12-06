pub(crate) mod bsgs;

use ark_ff::PrimeField;
use std::sync::Arc;

pub trait SquareMatrix<F: PrimeField>: Clone {
    fn dimension(&self) -> usize;
    fn get(&self, row: usize, col: usize) -> F;
    fn set_row_offset(&mut self, offset: usize);
    fn set_col_offset(&mut self, offset: usize);
}

impl<F: PrimeField> SquareMatrix<F> for Vec<Vec<F>> {
    fn dimension(&self) -> usize {
        self.len()
    }

    fn get(&self, row: usize, col: usize) -> F {
        self[row][col]
    }

    fn set_row_offset(&mut self, _offset: usize) {
        panic!("Not implemented");
    }

    fn set_col_offset(&mut self, _offset: usize) {
        panic!("Not implemented");
    }
}

// Meant to be cloned and used with different offsets
#[derive(Clone)]
pub struct SplittableMatrix<F: PrimeField> {
    matrix: Arc<Vec<Vec<F>>>,
    row_offset: usize,
    col_offset: usize,
}

impl<F: PrimeField> SplittableMatrix<F> {
    pub fn new(matrix: Vec<Vec<F>>) -> Self {
        Self {
            matrix: Arc::new(matrix),
            row_offset: 0,
            col_offset: 0,
        }
    }
}

impl<F: PrimeField> SquareMatrix<F> for SplittableMatrix<F> {
    fn dimension(&self) -> usize {
        self.matrix.len() - std::cmp::max(self.row_offset, self.col_offset)
    }

    fn get(&self, row: usize, col: usize) -> F {
        self.matrix[self.row_offset + row][self.col_offset + col]
    }

    fn set_row_offset(&mut self, offset: usize) {
        self.row_offset = offset;
    }

    fn set_col_offset(&mut self, offset: usize) {
        self.col_offset = offset;
    }
}
