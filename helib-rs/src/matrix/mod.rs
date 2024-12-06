pub(crate) mod bsgs;

use crate::{
    encoding::{galois::Galois, ntt::NTTProcessor},
    helib::error::Error,
    Ctxt,
};
use ark_ff::PrimeField;
use std::sync::Arc;

pub use bsgs::Bsgs;

pub fn plain_matrix_ctxt_vector<F: PrimeField, T: SquareMatrix<F>>(
    mat: &T,
    vec: &[Ctxt],
) -> Result<Vec<Ctxt>, Error> {
    let dim = mat.dimension();
    assert_eq!(dim, vec.len());
    let mut result = Vec::with_capacity(dim);

    for row in 0..dim {
        let mut res = vec[0].ctxt_mul_by_field_element(mat.get(row, 0))?;
        for (col, c) in vec.iter().enumerate().skip(1) {
            let tmp = c.ctxt_mul_by_field_element(mat.get(row, col))?;
            res.ctxt_add_inplace(&tmp)?;
        }
        result.push(res);
    }

    Ok(result)
}

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

#[derive(Clone)]
pub struct FFTMatrix<F: PrimeField> {
    n: usize,
    pow_table: Arc<Vec<F>>,
    row_offset: usize,
    col_offset: usize,
}

impl<F: PrimeField> FFTMatrix<F> {
    pub fn new(n: usize, root: F) -> Self {
        assert_eq!(root.pow([n as u64]), F::one());
        let pow_table = NTTProcessor::create_pow_table(2 * n, root);
        Self {
            n,
            pow_table: Arc::new(pow_table),
            row_offset: 0,
            col_offset: 0,
        }
    }

    pub fn get_groth16_root(n: usize) -> F {
        assert!(n.is_power_of_two());
        let (_, roots) = Galois::get_groth16_roots_of_unity();
        roots[n.ilog2() as usize]
    }

    pub fn get_minimal_root(n: usize) -> F {
        assert!(n.is_power_of_two());
        Galois::get_minimal_primitive_n_root_of_unity(n).expect("Root found")
    }
}

impl<F: PrimeField> SquareMatrix<F> for FFTMatrix<F> {
    fn dimension(&self) -> usize {
        self.n - std::cmp::max(self.row_offset, self.col_offset)
    }

    fn get(&self, row: usize, col: usize) -> F {
        let col = self.col_offset + col;
        let row = self.row_offset + row;
        let power = row * col;
        self.pow_table[power % self.n]
    }

    fn set_row_offset(&mut self, offset: usize) {
        self.row_offset = offset;
    }

    fn set_col_offset(&mut self, offset: usize) {
        self.col_offset = offset;
    }
}

#[derive(Clone)]
pub struct IFFTMatrix<F: PrimeField> {
    n: usize,
    pow_table: Arc<Vec<F>>,
    row_offset: usize,
    col_offset: usize,
}

impl<F: PrimeField> IFFTMatrix<F> {
    pub fn new(n: usize, root: F) -> Self {
        assert_eq!(root.pow([n as u64]), F::one());
        let inv_root = root.inverse().expect("mod inverse not found");
        let mut pow_table = NTTProcessor::create_pow_table(2 * n, inv_root);
        let n_inv = F::from(n as u64).inverse().expect("inverse not found");

        for p in pow_table.iter_mut() {
            *p *= n_inv;
        }

        Self {
            n,
            pow_table: Arc::new(pow_table),
            row_offset: 0,
            col_offset: 0,
        }
    }

    pub fn get_groth16_root(n: usize) -> F {
        FFTMatrix::get_groth16_root(n)
    }

    pub fn get_minimal_root(n: usize) -> F {
        FFTMatrix::get_minimal_root(n)
    }
}

impl<F: PrimeField> SquareMatrix<F> for IFFTMatrix<F> {
    fn dimension(&self) -> usize {
        self.n - std::cmp::max(self.row_offset, self.col_offset)
    }

    fn get(&self, row: usize, col: usize) -> F {
        let col = self.col_offset + col;
        let row = self.row_offset + row;
        let power = row * col;
        self.pow_table[power % self.n]
    }

    fn set_row_offset(&mut self, offset: usize) {
        self.row_offset = offset;
    }

    fn set_col_offset(&mut self, offset: usize) {
        self.col_offset = offset;
    }
}
