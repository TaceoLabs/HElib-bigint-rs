use crate::{
    encoding::batch_encoder::BatchEncoder,
    helib::{error::Error, galois_engine::GaloisEngine},
    Ctxt, EncodedPtxt,
};
use ark_ff::PrimeField;

use super::SquareMatrix;

pub struct Bsgs {}

impl Bsgs {
    fn babystep_giantstep_inner(
        ctxt: &mut Ctxt,
        encoded_diags: &[EncodedPtxt],
        galois_engine: &GaloisEngine,
        n1: usize,
        n2: usize,
        slots: usize,
    ) -> Result<(), Error> {
        let dim = encoded_diags.len();
        assert_eq!(dim, n1 * n2);
        // prepare for non-full-packed rotations
        if slots != dim << 1 {
            let mut state_rot = ctxt.ctxt_clone()?;
            // Here we loose tons of noise budget...
            galois_engine.rotate_ctxt(&mut state_rot, dim as i32)?;
            ctxt.ctxt_add_inplace(&state_rot)?;
        }

        let mut outer_sum = Ctxt::empty_pointer();

        // prepare rotations
        let mut rot = Vec::with_capacity(n1);
        rot.push(ctxt.ctxt_clone()?);
        for j in 1..n1 {
            let mut tmp = rot[j - 1].ctxt_clone()?;
            galois_engine.rotate_ctxt(&mut tmp, -1)?;
            rot.push(tmp);
        }

        for k in 0..n2 {
            let mut inner_sum = rot[0].ctxt_mul_by_packed_constant(&encoded_diags[k * n1])?;
            for j in 1..n1 {
                let tmp = rot[j].ctxt_mul_by_packed_constant(&encoded_diags[k * n1 + j])?;
                inner_sum.ctxt_add_inplace(&tmp)?;
            }

            if k == 0 {
                outer_sum = inner_sum;
            } else {
                galois_engine.rotate_ctxt(&mut inner_sum, -((k * n1) as i32))?;
                outer_sum.ctxt_add_inplace(&inner_sum)?;
            }
        }
        *ctxt = outer_sum;
        Ok(())
    }

    pub fn babystep_giantstep<F: PrimeField, T: SquareMatrix<F>>(
        ctxt: &mut Ctxt,
        matrix: &T,
        batch_encoder: &BatchEncoder<F>,
        galois_engine: &GaloisEngine,
        n1: usize,
        n2: usize,
    ) -> Result<(), Error> {
        let encoded = Self::encode_one_matrix(matrix, batch_encoder, n1, n2)?;
        Self::babystep_giantstep_inner(
            ctxt,
            &encoded,
            galois_engine,
            n1,
            n2,
            batch_encoder.slot_count(),
        )
    }

    pub fn babystep_giantstep_two_matrices<F: PrimeField, T: SquareMatrix<F>>(
        ctxt: &mut Ctxt,
        matrix1: &T,
        matrix2: &T,
        batch_encoder: &BatchEncoder<F>,
        galois_engine: &GaloisEngine,
        n1: usize,
        n2: usize,
    ) -> Result<(), Error> {
        let encoded = Self::encode_two_matrices(matrix1, matrix2, batch_encoder, n1, n2)?;
        Self::babystep_giantstep_inner(
            ctxt,
            &encoded,
            galois_engine,
            n1,
            n2,
            batch_encoder.slot_count(),
        )
    }

    fn encode_one_matrix<F: PrimeField, T: SquareMatrix<F>>(
        matrix: &T,
        batch_encoder: &BatchEncoder<F>,
        n1: usize,
        n2: usize,
    ) -> Result<Vec<EncodedPtxt>, Error> {
        let dim = n1 * n2;
        assert!(dim <= matrix.dimension());
        let slots = batch_encoder.slot_count();
        let halfslots = slots >> 1;
        assert!(dim << 1 == slots || dim << 2 < slots);
        assert_eq!(dim, n1 * n2);

        let mut encoded = Vec::with_capacity(dim);

        for i in 0..dim {
            let k = i / n1;
            let mut diag = Vec::with_capacity(halfslots);

            for j in 0..dim {
                diag.push(matrix.get(j, (j + dim - i) % dim));
            }
            // rotate:
            if k != 0 {
                diag.rotate_left(k * n1);
            }
            // prepare for non-full-packed rotations
            if slots != dim << 1 {
                diag.resize(halfslots, F::zero());
                for index in 0..k * n1 {
                    let index_src = dim - 1 - index;
                    let index_des = halfslots - 1 - index;
                    diag[index_des] = diag[index_src];
                    diag[index_src] = F::zero();
                }
            }
            let enc = EncodedPtxt::encode(&diag, batch_encoder)?;
            encoded.push(enc);
        }
        Ok(encoded)
    }

    fn encode_two_matrices<F: PrimeField, T: SquareMatrix<F>>(
        matrix1: &T,
        matrix2: &T,
        batch_encoder: &BatchEncoder<F>,
        n1: usize,
        n2: usize,
    ) -> Result<Vec<EncodedPtxt>, Error> {
        let dim = n1 * n2;
        assert!(dim <= matrix1.dimension());
        assert!(dim <= matrix2.dimension());
        let slots = batch_encoder.slot_count();
        let halfslots = slots >> 1;
        assert!(dim << 1 == slots || dim << 2 < slots);

        let mut encoded = Vec::with_capacity(dim);

        for i in 0..dim {
            let k = i / n1;
            let mut diag = Vec::with_capacity(slots);
            let mut tmp = Vec::with_capacity(dim);

            for j in 0..dim {
                diag.push(matrix1.get(j, (j + dim - i) % dim));
                tmp.push(matrix2.get(j, (j + dim - i) % dim));
            }
            // rotate:
            if k != 0 {
                diag.rotate_left(k * n1);
                tmp.rotate_left(k * n1);
            }
            // prepare for non-full-packed rotations
            if slots != dim << 1 {
                diag.resize(halfslots, F::zero());
                tmp.resize(halfslots, F::zero());
                for index in 0..k * n1 {
                    let index_src = dim - 1 - index;
                    let index_des = halfslots - 1 - index;
                    diag[index_des] = diag[index_src];
                    tmp[index_des] = tmp[index_src];
                    diag[index_src] = F::zero();
                    tmp[index_src] = F::zero();
                }
            }
            diag.resize(slots, F::zero());
            diag[halfslots..slots].copy_from_slice(&tmp[..(slots - halfslots)]);
            let enc = EncodedPtxt::encode(&diag, batch_encoder)?;
            encoded.push(enc);
        }
        Ok(encoded)
    }

    pub fn fully_packed_bsgs<F: PrimeField, T: SquareMatrix<F>>(
        ctxt: &mut Ctxt,
        matrix: &T,
        batch_encoder: &BatchEncoder<F>,
        galois_engine: &GaloisEngine,
    ) -> Result<(), Error> {
        let dim = matrix.dimension();
        let slots = batch_encoder.slot_count();
        assert_eq!(dim, slots);
        let dim_half = dim >> 1;
        let n2 = 1 << (dim_half.ilog2() >> 1);
        let n1 = dim_half / n2;

        // Strategy: Split M = [M1, M2] [M3, M4] and v = [v1, v2], then result r = [r1, r2]  is computed as (M1*v1 + M2*v2, M3*v1 + M4*v2)
        let mut ctxt2 = ctxt.ctxt_clone()?;

        // First half: M1*v1 + M4*v2
        let mat1 = matrix.clone();
        let mut mat4 = matrix.clone();
        mat4.set_col_offset(dim_half);
        mat4.set_row_offset(dim_half);
        Self::babystep_giantstep_two_matrices(
            ctxt,
            &mat1,
            &mat4,
            batch_encoder,
            galois_engine,
            n1,
            n2,
        )?;

        // Second half: M3*v1 + M2*v2
        let mut mat3 = mat1;
        mat3.set_row_offset(dim_half);
        let mut mat2 = mat4;
        mat2.set_row_offset(0);
        mat2.set_col_offset(dim_half);
        Self::babystep_giantstep_two_matrices(
            &mut ctxt2,
            &mat3,
            &mat2,
            batch_encoder,
            galois_engine,
            n1,
            n2,
        )?;

        // Combine results: ctxt + swapped(ctxt2)
        galois_engine.rotate_ctxt_columns(&mut ctxt2)?;
        ctxt.ctxt_add_inplace(&ctxt2)
    }

    pub fn bsgs_indices(n1: usize, n2: usize, slots: usize) -> Vec<i32> {
        let mut result = Vec::new();

        let dim = n1 * n2;
        if slots != dim << 1 {
            result.reserve(n2 + 1);
            result.push(dim as i32);
        } else {
            result.reserve(n2);
        }

        result.push(-1);
        for k in 1..n2 {
            result.push(-((k * n1) as i32));
        }

        result
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use crate::{helib::CLong, matrix::SplittableMatrix, Context, PubKey, SecKey, ZZ};
    use ark_ff::{UniformRand, Zero};
    use rand::thread_rng;

    const N: usize = 16384;
    const M: usize = 2 * N;
    const BITS: CLong = 850;

    fn plain_mat_vec<F: PrimeField>(matrix: &[Vec<F>], vec: &[F]) -> Vec<F> {
        assert_eq!(matrix.len(), vec.len());
        matrix.iter().for_each(|i| assert_eq!(i.len(), vec.len()));
        matrix
            .iter()
            .map(|row| row.iter().zip(vec).map(|(a, b)| *a * *b).sum())
            .collect()
    }

    #[test]
    fn bsgs_test() {
        // let dim = N >> 1;
        // let n2 = 1 << (dim.ilog2() >> 1);
        // let n1 = dim / n2;
        let dim = 200;
        let n1 = 20;
        let n2 = 10;
        let mut rng = thread_rng();

        let vec = (0..dim)
            .map(|_| ark_bn254::Fr::rand(&mut rng))
            .collect::<Vec<_>>();
        let mat = (0..dim)
            .map(|_| {
                (0..dim)
                    .map(|_| ark_bn254::Fr::rand(&mut rng))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let expected = plain_mat_vec(&mat, &vec);

        // HE
        let p = ZZ::char::<ark_bn254::Fr>().unwrap();
        let context = Context::build(M as CLong, &p, BITS).unwrap();
        let mut galois = GaloisEngine::build(M as CLong).unwrap();
        let seckey = SecKey::build(&context).unwrap();
        let pubkey = PubKey::from_seckey(&seckey).unwrap();
        let batch_encoder = BatchEncoder::new(N);

        for index in Bsgs::bsgs_indices(n1, n2, N) {
            galois.generate_key_for_step(&seckey, index).unwrap();
        }

        let encoded = EncodedPtxt::encode(&vec, &batch_encoder).unwrap();
        let mut ctxt = pubkey.packed_encrypt(&encoded).unwrap();

        Bsgs::babystep_giantstep(&mut ctxt, &mat, &batch_encoder, &galois, n1, n2).unwrap();

        let decrypted = seckey.packed_decrypt(&ctxt).unwrap();
        let decoded = decrypted.decode(&batch_encoder).unwrap();
        assert_eq!(expected, &decoded[..dim]);
    }

    #[test]
    fn bsgs_two_mats_test() {
        // let dim = N >> 1;
        // let n2 = 1 << (dim.ilog2() >> 1);
        // let n1 = dim / n2;
        let dim = 200;
        let n1 = 20;
        let n2 = 10;
        let mut rng = thread_rng();

        let vec1 = (0..dim)
            .map(|_| ark_bn254::Fr::rand(&mut rng))
            .collect::<Vec<_>>();
        let vec2 = (0..dim)
            .map(|_| ark_bn254::Fr::rand(&mut rng))
            .collect::<Vec<_>>();
        let mat1 = (0..dim)
            .map(|_| {
                (0..dim)
                    .map(|_| ark_bn254::Fr::rand(&mut rng))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let mat2 = (0..dim)
            .map(|_| {
                (0..dim)
                    .map(|_| ark_bn254::Fr::rand(&mut rng))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let expected1 = plain_mat_vec(&mat1, &vec1);
        let expected2 = plain_mat_vec(&mat2, &vec2);

        // HE
        let p = ZZ::char::<ark_bn254::Fr>().unwrap();
        let context = Context::build(M as CLong, &p, BITS).unwrap();
        let mut galois = GaloisEngine::build(M as CLong).unwrap();
        let seckey = SecKey::build(&context).unwrap();
        let pubkey = PubKey::from_seckey(&seckey).unwrap();
        let batch_encoder = BatchEncoder::new(N);

        for index in Bsgs::bsgs_indices(n1, n2, N) {
            galois.generate_key_for_step(&seckey, index).unwrap();
        }

        let slots = batch_encoder.slot_count();
        let halfslots = slots >> 1;
        let mut vec = Vec::with_capacity(halfslots + dim);
        vec.extend_from_slice(&vec1);
        vec.resize(halfslots, ark_bn254::Fr::zero());
        vec.extend_from_slice(&vec2);

        let encoded = EncodedPtxt::encode(&vec, &batch_encoder).unwrap();
        let mut ctxt = pubkey.packed_encrypt(&encoded).unwrap();

        Bsgs::babystep_giantstep_two_matrices(
            &mut ctxt,
            &mat1,
            &mat2,
            &batch_encoder,
            &galois,
            n1,
            n2,
        )
        .unwrap();

        let decrypted = seckey.packed_decrypt(&ctxt).unwrap();
        let decoded = decrypted.decode(&batch_encoder).unwrap();
        assert_eq!(expected1, &decoded[..dim]);
        assert_eq!(expected2, &decoded[halfslots..halfslots + dim]);
    }

    #[test]
    #[ignore]
    fn fully_packed_bsgs_test() {
        let dim = N;
        let dim_half = dim >> 1;
        let n2 = 1 << (dim_half.ilog2() >> 1);
        let n1 = dim_half / n2;
        let mut rng = thread_rng();

        let vec = (0..dim)
            .map(|_| ark_bn254::Fr::rand(&mut rng))
            .collect::<Vec<_>>();
        let mat = (0..dim)
            .map(|_| {
                (0..dim)
                    .map(|_| ark_bn254::Fr::rand(&mut rng))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let expected = plain_mat_vec(&mat, &vec);

        // HE
        let p = ZZ::char::<ark_bn254::Fr>().unwrap();
        let context = Context::build(M as CLong, &p, BITS).unwrap();
        let mut galois = GaloisEngine::build(M as CLong).unwrap();
        let seckey = SecKey::build(&context).unwrap();
        let pubkey = PubKey::from_seckey(&seckey).unwrap();
        let batch_encoder = BatchEncoder::new(N);

        for index in Bsgs::bsgs_indices(n1, n2, N) {
            galois.generate_key_for_step(&seckey, index).unwrap();
        }
        galois.generate_key_for_step(&seckey, 0).unwrap(); // Column swap

        let encoded = EncodedPtxt::encode(&vec, &batch_encoder).unwrap();
        let mut ctxt = pubkey.packed_encrypt(&encoded).unwrap();

        let mat = SplittableMatrix::new(mat);
        Bsgs::fully_packed_bsgs(&mut ctxt, &mat, &batch_encoder, &galois).unwrap();

        let decrypted = seckey.packed_decrypt(&ctxt).unwrap();
        let decoded = decrypted.decode(&batch_encoder).unwrap();
        assert_eq!(expected, &decoded[..dim]);
    }
}
