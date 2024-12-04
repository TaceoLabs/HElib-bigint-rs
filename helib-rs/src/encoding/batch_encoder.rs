use super::{galois::Galois, ntt::NTTProcessor};
use ark_ff::PrimeField;

struct BatchEncoder<F: PrimeField> {
    n: usize,
    index_map: Vec<usize>,
    ntt_proc: NTTProcessor<F>,
}

impl<F: PrimeField> BatchEncoder<F> {
    pub(crate) fn new(n: usize) -> Self {
        let root = Galois::get_minimal_primitive_n_root_of_unity(2 * n).expect("no root found");
        Self {
            n,
            index_map: Self::populate_index_map(n),
            ntt_proc: NTTProcessor::new_negacylic(n, root),
        }
    }

    pub(crate) fn populate_index_map(slots: usize) -> Vec<usize> {
        assert!(slots.is_power_of_two());

        let mut index_map = vec![0usize; slots];

        let row_size = slots >> 1;
        let group_size = slots << 1;

        let gen: usize = Galois::GENERATOR;
        let mut pos: usize = 1;

        for i in 0..row_size {
            // position in normal bit order
            let index1 = (pos - 1) >> 1;
            let index2 = (group_size - pos - 1) >> 1;

            index_map[i] = index1;
            index_map[i + row_size] = index2;

            // next primitive root
            pos = (pos * gen) % group_size;
        }

        index_map
    }

    pub(crate) fn rotate_encoded(input: &[F], index: i32) -> Vec<F> {
        Galois::automorphism(input, Galois::get_elt_from_step(input.len(), index))
    }

    pub(crate) fn encode(&self, input: &[F]) -> Vec<F> {
        let mut encoded = vec![F::zero(); self.n];

        for (i, val) in input.iter().enumerate() {
            encoded[self.index_map[i]] = *val;
        }

        self.ntt_proc.inverse_transform_inplace(&mut encoded);
        self.ntt_proc.negacylcic_postprocess(&mut encoded);
        encoded
    }

    pub(crate) fn decode(&self, input: &[F]) -> Vec<F> {
        let mut transformed = input.to_vec();
        self.ntt_proc.negacylcic_preprocess(&mut transformed);
        self.ntt_proc.transform_inplace(&mut transformed);

        (0..self.n)
            .map(|i| transformed[self.index_map[i]])
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::encoding::{negacyclic_naive_mult, rotate_plain};
    use ark_ff::UniformRand;
    use rand::{thread_rng, Rng};

    const NUM_TRIALS: usize = 10;
    static N: usize = 1024;

    #[test]
    fn encode_decode_test() {
        let input: Vec<_> = (0..N).map(|i| ark_bn254::Fr::from(i as u64)).collect();
        let encoder = BatchEncoder::new(N);
        let encode = encoder.encode(&input);
        let result = encoder.decode(&encode);

        assert_eq!(input, result);
    }

    #[test]
    fn batch_add_test() {
        let encoder = BatchEncoder::new(N);

        let mut rng = thread_rng();
        for _ in 0..NUM_TRIALS {
            let input1: Vec<_> = (0..N).map(|_| ark_bn254::Fr::rand(&mut rng)).collect();
            let input2: Vec<_> = (0..N).map(|_| ark_bn254::Fr::rand(&mut rng)).collect();

            let result_plain: Vec<_> = input1
                .iter()
                .zip(input2.iter())
                .map(|(el1, el2)| el1 + el2)
                .collect();

            let encode1 = encoder.encode(&input1);
            let encode2 = encoder.encode(&input2);
            let added: Vec<_> = encode1
                .iter()
                .zip(encode2.iter())
                .map(|(el1, el2)| el1 + el2)
                .collect();

            let result = encoder.decode(&added);

            assert_eq!(result_plain, result);
        }
    }

    #[test]
    fn batch_mul_test() {
        let encoder = BatchEncoder::new(N);

        let mut rng = thread_rng();
        for _ in 0..NUM_TRIALS {
            let input1: Vec<_> = (0..N).map(|_| ark_bn254::Fr::rand(&mut rng)).collect();
            let input2: Vec<_> = (0..N).map(|_| ark_bn254::Fr::rand(&mut rng)).collect();

            let result_plain: Vec<_> = input1
                .iter()
                .zip(input2.iter())
                .map(|(el1, el2)| el1 * el2)
                .collect();

            let encode1 = encoder.encode(&input1);
            let encode2 = encoder.encode(&input2);

            let multiplied = negacyclic_naive_mult(&encode1, &encode2);

            let result = encoder.decode(&multiplied);

            assert_eq!(result_plain, result);
        }
    }

    #[test]
    fn rotate_test() {
        let input: Vec<_> = (0..N).map(|i| ark_bn254::Fr::from(i as u64)).collect();
        let encoder = BatchEncoder::new(N);
        let encode = encoder.encode(&input);
        let n2 = (N >> 1) as i32;

        let mut rng = thread_rng();
        for _ in 0..NUM_TRIALS {
            let index = rng.gen_range(-n2 + 1..n2);
            let result_plain = rotate_plain(&input, index);
            let result_encoded = BatchEncoder::rotate_encoded(&encode, index);

            let result = encoder.decode(&result_encoded);
            assert_eq!(result_plain, result);
        }
    }
}
