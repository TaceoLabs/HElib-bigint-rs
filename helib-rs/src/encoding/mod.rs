pub(crate) mod ark_ntt;
pub(crate) mod batch_encoder;
pub(crate) mod galois;
pub(crate) mod ntt;

pub(crate) fn reverse_bits32(input: u32) -> u32 {
    let mut output = ((input & 0xaaaaaaaa) >> 1) | ((input & 0x55555555) << 1);
    output = ((output & 0xcccccccc) >> 2) | ((output & 0x33333333) << 2);
    output = ((output & 0xf0f0f0f0) >> 4) | ((output & 0x0f0f0f0f) << 4);
    output = ((output & 0xff00ff00) >> 8) | ((output & 0x00ff00ff) << 8);
    output.rotate_left(16)
}

pub(crate) fn reverse_bits(input: u64) -> u64 {
    reverse_bits32((input >> 32) as u32) as u64 | (reverse_bits32(input as u32) as u64) << 32
}

pub(crate) fn reverse_n_bits(input: u64, num_bits: u64) -> u64 {
    reverse_bits(input) >> (64 - num_bits)
}

#[cfg(test)]
use ark_ff::PrimeField;

#[cfg(test)]
fn negacyclic_naive_mult<F: PrimeField>(a: &[F], b: &[F]) -> Vec<F> {
    assert!(a.len() == b.len());
    let mut result = vec![F::zero(); a.len()];
    for i in 0..a.len() {
        let mut acc = F::zero();
        for j in 0..=i {
            acc += a[j] * b[i - j];
        }
        for j in (i + 1)..a.len() {
            let sub = a[j] * b[a.len() + i - j];
            acc -= sub; // negacylcic
        }
        result[i] = acc;
    }
    result
}

#[cfg(test)]
pub(crate) fn cyclic_naive_mult<F: PrimeField>(a: &[F], b: &[F]) -> Vec<F> {
    assert!(a.len() == b.len());
    let mut result = vec![F::zero(); a.len()];
    for i in 0..a.len() {
        let mut acc = F::zero();
        for j in 0..=i {
            acc += a[j] * b[i - j];
        }
        for j in (i + 1)..a.len() {
            let sub = a[j] * b[a.len() + i - j];
            acc += sub; // cyclic
        }
        result[i] = acc;
    }
    result
}

#[cfg(test)]
pub(crate) fn rotate_plain<F: PrimeField>(input: &[F], i: i32) -> Vec<F> {
    let n = input.len();
    assert!(n.is_power_of_two());
    let row_size = n / 2;
    let i_abs = i.unsigned_abs() as usize;
    assert!(i_abs < row_size);
    let step = if i < 0 { row_size - i_abs } else { i_abs };
    let mut result = vec![F::zero(); n];

    for j in 0..row_size {
        if step == 0 {
            result[j] = input[row_size + j];
            result[row_size + j] = input[j];
        } else {
            result[j] = input[(j + step) % row_size];
            result[row_size + j] = input[(j + step) % row_size + row_size];
        }
    }

    result
}
