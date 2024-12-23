use ark_ff::PrimeField;
use ark_poly::{EvaluationDomain, Radix2EvaluationDomain};

pub struct NTTProcessor<F: PrimeField> {
    n: usize,
    n_inv: F,
    root: F,
    root_inverse: F,
    pow_table: Vec<F>,
    inv_pow_table: Vec<F>,
    ark_ff_domain: Radix2EvaluationDomain<F>,
}

impl<F: PrimeField> NTTProcessor<F> {
    pub fn new(n: usize, root: F) -> Self {
        assert!(n.is_power_of_two());
        let root_inverse = root.inverse().expect("mod inverse not found");
        let n_inv = F::from(n as u64).inverse().expect("inverse not found");

        let mut ark_ff_domain =
            Radix2EvaluationDomain::<F>::new(n).expect("Can create ark_ff_domain");
        ark_ff_domain.group_gen = root;
        ark_ff_domain.group_gen_inv = root_inverse;

        NTTProcessor {
            n,
            n_inv,
            root,
            root_inverse,
            pow_table: NTTProcessor::create_pow_table(n, root),
            inv_pow_table: NTTProcessor::create_pow_table(n, root_inverse),
            ark_ff_domain,
        }
    }

    pub fn new_negacylic(n: usize, root: F) -> Self {
        assert!(n.is_power_of_two());
        let root_squared = root.square();
        let root_inverse = root.inverse().expect("mod inverse not found");
        let n_inv = F::from(n as u64).inverse().expect("inverse not found");
        let root_squared_inverse = root_squared.inverse().expect("mod inverse not found");

        let mut ark_ff_domain =
            Radix2EvaluationDomain::<F>::new(n).expect("Can create ark_ff_domain");
        ark_ff_domain.group_gen = root_squared;
        ark_ff_domain.group_gen_inv = root_squared_inverse;

        NTTProcessor {
            n,
            n_inv,
            root,
            root_inverse,
            pow_table: NTTProcessor::create_pow_table(n, root_squared),
            inv_pow_table: NTTProcessor::create_pow_table(n, root_squared_inverse),
            ark_ff_domain,
        }
    }

    pub fn negacylcic_preprocess_two(&self, a: &mut [F], b: &mut [F]) {
        debug_assert_eq!(a.len(), self.n);
        debug_assert_eq!(b.len(), self.n);
        let mut tmp = F::one();
        for (a, b) in a.iter_mut().zip(b.iter_mut()) {
            *a *= tmp;
            *b *= tmp;
            tmp *= self.root;
        }
    }

    pub fn negacylcic_preprocess(&self, a: &mut [F]) {
        debug_assert_eq!(a.len(), self.n);
        let mut tmp = F::one();
        for a in a.iter_mut() {
            *a *= tmp;
            tmp *= self.root;
        }
    }

    pub fn negacylcic_postprocess(&self, a: &mut [F]) {
        debug_assert_eq!(a.len(), self.n);
        let mut tmp = F::one();
        for a in a.iter_mut() {
            *a *= tmp;
            tmp *= self.root_inverse;
        }
    }

    pub(crate) fn create_pow_table(n: usize, root: F) -> Vec<F> {
        let mut table = Vec::with_capacity(n >> 1);
        let mut tmp = F::one();
        for _ in 0..n >> 1 {
            table.push(tmp);
            tmp *= root;
        }
        table
    }

    pub fn ntt(&self, input: &[F]) -> Vec<F> {
        let mut output = input.to_vec();
        self.ntt_inplace(&mut output);
        output
    }

    pub fn ntt_inplace(&self, input: &mut [F]) {
        assert_eq!(input.len(), self.n);
        let levels = self.n.ilog2();
        for i in 0..self.n {
            let j = super::reverse_n_bits(i as u64, levels as u64) as usize;
            if j > i {
                input.swap(i, j);
            }
        }

        let mut size = 2;
        while size <= self.n {
            let halfsize = size >> 1;
            let tablestep = self.n / size;
            for i in (0..self.n).step_by(size) {
                let mut k = 0;
                for j in i..i + halfsize {
                    let l = j + halfsize;
                    let left = input[j];
                    let right = input[l] * self.pow_table[k];

                    input[j] = left + right;
                    input[l] = left - right;

                    k += tablestep;
                }
            }
            size *= 2;
        }
    }

    pub fn intt(&self, input: &[F]) -> Vec<F> {
        let mut output = input.to_vec();
        self.intt_inplace(&mut output);
        output
    }

    pub fn intt_inplace(&self, input: &mut [F]) {
        assert_eq!(input.len(), self.n);
        let levels = self.n.ilog2();

        for i in 0..self.n {
            let j = super::reverse_n_bits(i as u64, levels as u64) as usize;
            if j > i {
                input.swap(i, j);
            }
        }

        let mut size = 2;
        while size <= self.n {
            let halfsize = size >> 1;
            let tablestep = self.n / size;
            for i in (0..self.n).step_by(size) {
                let mut k = 0;
                for j in i..i + halfsize {
                    let l = j + halfsize;
                    let left = input[j];
                    let right = input[l] * self.inv_pow_table[k];

                    input[j] = left + right;
                    input[l] = left - right;

                    k += tablestep;
                }
            }
            size *= 2;
        }

        input.iter_mut().for_each(|el| *el *= self.n_inv);
    }

    pub fn fft(&self, input: &[F]) -> Vec<F> {
        self.ark_ff_domain.fft(input)
    }

    pub fn fft_inplace(&self, input: &mut Vec<F>) {
        self.ark_ff_domain.fft_in_place(input)
    }

    pub fn ifft(&self, input: &[F]) -> Vec<F> {
        self.ark_ff_domain.ifft(input)
    }

    pub fn ifft_inplace(&self, input: &mut Vec<F>) {
        self.ark_ff_domain.ifft_in_place(input)
    }
}

#[cfg(test)]
mod ntt_test {
    use super::*;
    use crate::encoding::{cyclic_naive_mult, galois::Galois, negacyclic_naive_mult};
    use ark_ff::{UniformRand, Zero};
    use rand::thread_rng;

    const NUM_TRIALS: usize = 5;
    const N: usize = 4096;

    #[test]
    fn ntt_is_bijective() {
        let root = Galois::get_minimal_primitive_n_root_of_unity(N).expect("no root found!");
        let ntt_proc = NTTProcessor::new(N, root);

        let mut rng = thread_rng();
        for _ in 0..NUM_TRIALS {
            let poly: Vec<_> = (0..N).map(|_| ark_bn254::Fr::rand(&mut rng)).collect();
            let b = ntt_proc.ntt(&poly);
            let c = ntt_proc.intt(&b);

            assert_eq!(poly, c);
        }
    }
    #[test]
    fn ntt_constant() {
        let root = Galois::get_minimal_primitive_n_root_of_unity(N).expect("no root found!");
        let ntt_proc = NTTProcessor::new(N, root);

        let mut rng = thread_rng();
        for _ in 0..NUM_TRIALS {
            let mu = ark_bn254::Fr::rand(&mut rng);
            let mut poly = vec![ark_bn254::Fr::zero(); N];
            poly[0] = mu * ark_bn254::Fr::from(N as u64);
            let b = vec![mu; N];

            let c = ntt_proc.ntt(&b);

            assert_eq!(poly, c);
        }
    }

    #[test]
    fn negacyclic_ntt_mult_test() {
        let root = Galois::get_minimal_primitive_n_root_of_unity(2 * N).expect("no root found!");
        let ntt_proc = NTTProcessor::new_negacylic(N, root);

        let mut rng = thread_rng();
        for _ in 0..NUM_TRIALS {
            let mut a: Vec<_> = (0..N).map(|_| ark_bn254::Fr::rand(&mut rng)).collect();
            let mut b: Vec<_> = (0..N).map(|_| ark_bn254::Fr::rand(&mut rng)).collect();
            let naive = negacyclic_naive_mult(&a, &b);

            ntt_proc.negacylcic_preprocess_two(a.as_mut(), b.as_mut());
            let a_ntt = ntt_proc.ntt(&a);
            let b_ntt = ntt_proc.ntt(&b);

            let result_ntt: Vec<_> = a_ntt
                .iter()
                .zip(b_ntt.iter())
                .map(|(a_, b_)| *a_ * *b_)
                .collect();

            let mut result = ntt_proc.intt(&result_ntt);
            ntt_proc.negacylcic_postprocess(result.as_mut());

            assert_eq!(result, naive);
        }
    }

    #[test]
    fn cyclic_ntt_mult_test() {
        let root = Galois::get_minimal_primitive_n_root_of_unity(N).expect("no root found!"); // cyclic ntt
        let ntt_proc = NTTProcessor::new(N, root);

        let mut rng = thread_rng();
        for _ in 0..NUM_TRIALS {
            let a: Vec<_> = (0..N).map(|_| ark_bn254::Fr::rand(&mut rng)).collect();
            let b: Vec<_> = (0..N).map(|_| ark_bn254::Fr::rand(&mut rng)).collect();
            let naive = cyclic_naive_mult(&a, &b);

            let a_ntt = ntt_proc.ntt(&a);
            let b_ntt = ntt_proc.ntt(&b);

            let result_ntt: Vec<_> = a_ntt
                .iter()
                .zip(b_ntt.iter())
                .map(|(a_, b_)| *a_ * *b_)
                .collect();

            let result = ntt_proc.intt(&result_ntt);
            assert_eq!(result, naive);
        }
    }

    #[test]
    fn cyclic_ntt_vs_ark_fft() {
        let root = Galois::get_minimal_primitive_n_root_of_unity(N).expect("no root found!"); // cyclic ntt
        let ntt_proc = NTTProcessor::new(N, root);

        let mut rng = thread_rng();
        for _ in 0..NUM_TRIALS {
            let a: Vec<_> = (0..N).map(|_| ark_bn254::Fr::rand(&mut rng)).collect();

            let a_ntt = ntt_proc.ntt(&a);
            let a_fft = ntt_proc.fft(&a);
            assert_eq!(a_ntt, a_fft);
        }
    }

    #[test]
    fn cyclic_intt_vs_ark_ifft() {
        let root = Galois::get_minimal_primitive_n_root_of_unity(N).expect("no root found!"); // cyclic ntt
        let ntt_proc = NTTProcessor::new(N, root);

        let mut rng = thread_rng();
        for _ in 0..NUM_TRIALS {
            let a: Vec<_> = (0..N).map(|_| ark_bn254::Fr::rand(&mut rng)).collect();

            let a_ntt = ntt_proc.intt(&a);
            let a_fft = ntt_proc.ifft(&a);
            assert_eq!(a_ntt, a_fft);
        }
    }

    #[test]
    fn cyclic_ntt_vs_ark_fft_groth_root() {
        let (_, roots) = Galois::get_groth16_roots_of_unity();
        let ntt_proc = NTTProcessor::new(N, roots[N.ilog2() as usize]);

        let mut rng = thread_rng();
        for _ in 0..NUM_TRIALS {
            let a: Vec<_> = (0..N).map(|_| ark_bn254::Fr::rand(&mut rng)).collect();

            let a_ntt = ntt_proc.ntt(&a);
            let a_fft = ntt_proc.fft(&a);
            assert_eq!(a_ntt, a_fft);
        }
    }

    #[test]
    fn cyclic_intt_vs_ark_ifft_groth_root() {
        let (_, roots) = Galois::get_groth16_roots_of_unity();
        let ntt_proc = NTTProcessor::new(N, roots[N.ilog2() as usize]);

        let mut rng = thread_rng();
        for _ in 0..NUM_TRIALS {
            let a: Vec<_> = (0..N).map(|_| ark_bn254::Fr::rand(&mut rng)).collect();

            let a_ntt = ntt_proc.intt(&a);
            let a_fft = ntt_proc.ifft(&a);
            assert_eq!(a_ntt, a_fft);
        }
    }
}
