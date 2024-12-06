use ark_ff::{LegendreSymbol, One, PrimeField};
use num_bigint::BigUint;
use rand::thread_rng;

pub(crate) struct Galois {}

impl Galois {
    pub(crate) const GENERATOR: usize = 3;
    const MAX_ATTEMPTS: usize = 100;

    /// transforms rotation step to automorphism element
    pub(crate) fn get_elt_from_step(n: usize, step: i32) -> usize {
        let n2 = n << 1;
        let row_size = n >> 1;
        if step == 0 {
            return n2 - 1;
        }

        let sign = step < 0;
        let step_abs = step.unsigned_abs() as usize;

        assert!(step_abs < row_size, "step count too large");
        let step = if sign { row_size - step_abs } else { step_abs };

        let mut galois_element = 1;
        for _ in 0..step {
            galois_element = (galois_element * Self::GENERATOR) % n2;
        }
        galois_element
    }

    #[expect(dead_code)]
    pub(crate) fn get_elts_from_steps(n: usize, steps: &[i32]) -> Vec<usize> {
        steps
            .iter()
            .map(|step| Self::get_elt_from_step(n, *step))
            .collect()
    }

    /// plaintext prime p, for x^n/2 + 1
    pub(crate) fn get_primitive_n_root_of_unity<F: PrimeField>(n: usize) -> Option<F> {
        let p_biguint: BigUint = F::MODULUS.into();

        assert_eq!(&p_biguint % n, BigUint::one(), "p must be 1 mod {}", n);

        let group_size = (p_biguint - BigUint::one()) / n;
        let group_size_u64 = group_size.to_u64_digits();
        let n_pow = [(n >> 1) as u64];

        let mut rand = thread_rng();
        for _ in 0..Self::MAX_ATTEMPTS {
            let x = F::rand(&mut rand);
            let root = x.pow(&group_size_u64);
            let check = root.pow(n_pow) + F::one();
            if check.is_zero() {
                return Some(root);
            }
        }
        None
    }

    /// plaintext prime p, for x^n/2 + 1
    pub(crate) fn get_minimal_primitive_n_root_of_unity<F: PrimeField>(n: usize) -> Option<F> {
        let mut root = match Self::get_primitive_n_root_of_unity::<F>(n) {
            Some(i) => i,
            None => return None,
        };

        let gen = root.square();
        let mut current_gen = root;

        for _ in 0..n {
            if current_gen < root {
                root = current_gen;
            }
            current_gen *= gen;
        }

        Some(root)
    }

    pub(crate) fn get_groth16_roots_of_unity<F: PrimeField>() -> (F, Vec<F>) {
        let mut roots = vec![F::zero(); F::TWO_ADICITY as usize + 1];
        let mut q = F::one();
        while q.legendre() != LegendreSymbol::QuadraticNonResidue {
            q += F::one();
        }
        let z = q.pow(F::TRACE);
        roots[0] = z;
        for i in 1..roots.len() {
            roots[i] = roots[i - 1].square();
        }
        roots.reverse();
        (q, roots)
    }

    pub(crate) fn automorphism<F: PrimeField>(a: &[F], galois_elt: usize) -> Vec<F> {
        let n = a.len();
        assert!(n.is_power_of_two());
        assert!(galois_elt & 1 == 1);
        assert!(galois_elt < 2 * n);

        let log2n = n.ilog2();

        let mut result = a.to_vec();
        let mut index_raw = 0;
        for val in a.iter() {
            let index = index_raw % n;
            if (index_raw >> log2n) & 1 == 0 {
                result[index] = *val;
            } else {
                result[index] = -*val;
            }
            index_raw += galois_elt;
        }
        result
    }
}
