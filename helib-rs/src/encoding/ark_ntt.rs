use ark_ff::{FftField, LegendreSymbol, PrimeField};
use ark_poly::{EvaluationDomain, Radix2EvaluationDomain};

fn roots_of_unity<F: PrimeField + FftField>() -> (F, Vec<F>) {
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

pub(crate) fn fft_domain<F: PrimeField + FftField>(
    domain_size: usize,
) -> Radix2EvaluationDomain<F> {
    if domain_size & (domain_size - 1) != 0 || domain_size == 0 {
        panic!("domain size must be a power of 2 and non-zero");
    };
    let mut domain = Radix2EvaluationDomain::<F>::new(domain_size).unwrap();

    let (_, roots_of_unity) = roots_of_unity();
    let pow = usize::try_from(domain_size.ilog2()).expect("u32 fits into usize");

    // snarkjs and arkworks use different roots of unity to compute (i)fft.
    // therefore we compute the roots of unity by hand like snarkjs and
    // set the root of unity accordingly by hand
    domain.group_gen = roots_of_unity[pow];
    domain.group_gen_inv = domain.group_gen.inverse().expect("can compute inverse");

    domain
}
