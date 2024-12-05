use crate::{
    encoding::batch_encoder::BatchEncoder,
    helib::{error::Error, galois_engine::GaloisEngine},
    Ctxt, EncodedPtxt,
};
use ark_ff::PrimeField;

fn babystep_giantstep<F: PrimeField>(
    ctxt: &mut Ctxt,
    matrix: &[Vec<F>],
    batch_encoder: &BatchEncoder<F>,
    galois_engine: &GaloisEngine,
    n1: usize,
    n2: usize,
) -> Result<(), Error> {
    let dim = matrix.len();
    let slots = batch_encoder.slot_count();
    assert!(dim << 1 == slots || dim << 2 < slots);
    assert_eq!(dim, n1 * n2);

    let mut encoded = Vec::with_capacity(dim);

    for i in 0..dim {
        let k = i / n1;
        let mut diag = Vec::with_capacity(dim + k * n1);

        for (j, matrix) in matrix.iter().enumerate() {
            // for (auto j = 0ULL; j < matrix_dim; j++) {
            diag.push(matrix[(i + j) % dim]);
        }
        // rotate:
        if k != 0 {
            diag.rotate_left(dim - k * n1);
        }
        // prepare for non-full-packed rotations
        if slots != dim << 1 {
            for index in 0..k * n1 {
                diag.push(diag[index]);
                diag[index] = F::zero();
            }
        }
        let enc = EncodedPtxt::encode(&diag, batch_encoder)?;
        encoded.push(enc);
    }

    // prepare for non-full-packed rotations
    if slots != dim << 1 {
        let mut state_rot = ctxt.ctxt_clone()?;
        galois_engine.rotate_ctxt(&mut state_rot, -(dim as i32))?;
        ctxt.ctxt_add_inplace(&state_rot)?;
    }

    let mut outer_sum = Ctxt::empty_pointer();

    // prepare rotations
    let mut rot = Vec::with_capacity(n1);
    rot.push(ctxt.ctxt_clone()?);
    for j in 1..n1 {
        let mut tmp = rot[j - 1].ctxt_clone()?;
        galois_engine.rotate_ctxt(&mut tmp, 1)?;
        rot.push(tmp);
    }

    for k in 0..n2 {
        let mut inner_sum = rot[0].ctxt_mul_by_packed_constant(&encoded[k * n1])?;
        for j in 1..n1 {
            let tmp = rot[j].ctxt_mul_by_packed_constant(&encoded[k * n1 + j])?;
            inner_sum.ctxt_add_inplace(&tmp)?;
        }

        if k == 0 {
            outer_sum = inner_sum;
        } else {
            galois_engine.rotate_ctxt(&mut inner_sum, (k * n1) as i32)?;
            outer_sum.ctxt_add_inplace(&inner_sum)?;
        }
    }
    *ctxt = outer_sum;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{helib::CLong, Context, PubKey, SecKey, ZZ};
    use ark_ff::UniformRand;
    use rand::thread_rng;

    const N: usize = 16384;
    const M: usize = 2 * N;
    const BITS: CLong = 700;

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
        let dim = N >> 1;
        let n2 = 1 << (dim.ilog2() >> 1);
        let n1 = dim / n2;
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

        let encoded = EncodedPtxt::encode(&vec, &batch_encoder).unwrap();
        let mut ctxt = pubkey.packed_encrypt(&encoded).unwrap();

        babystep_giantstep(&mut ctxt, &mat, &batch_encoder, &galois, n1, n2).unwrap();

        let decrypted = seckey.packed_decrypt(&ctxt).unwrap();
        let decoded = decrypted.decode(&batch_encoder).unwrap();
        assert_eq!(expected, &decoded[..dim]);
    }
}
