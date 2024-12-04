// use ark_ff::PrimeField;

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
