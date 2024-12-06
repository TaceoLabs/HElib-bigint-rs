use ark_ff::PrimeField;
use helib_rs::{
    matrix::{Bsgs, FFTMatrix},
    BatchEncoder, CLong, Context, Ctxt, EncodedPtxt, Error, GaloisEngine, NTTProcessor, PubKey,
    SecKey, ZZ,
};
use rand::{thread_rng, Rng};
use std::process::ExitCode;

const HE_N: CLong = 1024;
const HE_M: CLong = 2 * HE_N;
const HE_BITS: CLong = 700;

struct HeContext<F: PrimeField> {
    _context: Context,
    seckey: SecKey,
    pubkey: PubKey,
    encoder: BatchEncoder<F>,
    galois: GaloisEngine,
}

impl<F: PrimeField> HeContext<F> {
    fn new(m: CLong, bits: CLong) -> Result<Self, Error> {
        let p = ZZ::char::<ark_bn254::Fr>()?;
        let context = Context::build(m, &p, bits)?;
        let galois = GaloisEngine::build(m)?;
        let seckey = SecKey::build(&context)?;
        let pubkey = PubKey::from_seckey(&seckey)?;
        let encoder = BatchEncoder::new(m);

        Ok(Self {
            _context: context,
            seckey,
            pubkey,
            encoder,
            galois,
        })
    }
}

fn random_vec<F: PrimeField, R: Rng>(size: usize, rng: &mut R) -> Vec<F> {
    (0..size).map(|_| F::rand(rng)).collect()
}

fn encrypt<F: PrimeField>(inputs: &[F], context: &HeContext<F>) -> Result<Vec<Ctxt>, Error> {
    let mut ctxts = Vec::with_capacity(inputs.len().div_ceil(HE_N as usize));
    for inp in inputs.chunks(HE_N as usize) {
        let encode = EncodedPtxt::encode(inp, &context.encoder)?;
        let ctxt = context.pubkey.packed_encrypt(&encode)?;
        ctxts.push(ctxt);
    }
    Ok(ctxts)
}

fn decrypt<F: PrimeField>(
    size: usize,
    ctxts: &[Ctxt],
    context: &HeContext<F>,
) -> Result<Vec<F>, Error> {
    let mut outputs = Vec::with_capacity(ctxts.len() * HE_N as usize);
    for ctxt in ctxts {
        let ptxt = context.seckey.packed_decrypt(ctxt)?;
        let output = ptxt.decode(&context.encoder)?;
        outputs.extend(output);
    }
    outputs.resize(size, F::zero());
    Ok(outputs)
}

fn packed_fft<F: PrimeField>(
    ctxt: &Ctxt,
    dim: usize,
    root: F,
    context: &mut HeContext<F>,
) -> Result<Ctxt, Error> {
    let n2 = 1 << (dim.ilog2() >> 1);
    let n1 = dim / n2;

    // Galois keys:
    for index in Bsgs::bsgs_indices(n1, n2, context.encoder.slot_count()) {
        context
            .galois
            .generate_key_for_step(&context.seckey, index)?;
    }

    // Actual FFT:
    let mat = FFTMatrix::new(dim, root);
    let mut result = ctxt.ctxt_clone()?;
    Bsgs::babystep_giantstep(&mut result, &mat, &context.encoder, &context.galois, n1, n2)?;

    Ok(result)
}

fn fully_packed_fft<F: PrimeField>(
    ctxt: &Ctxt,
    root: F,
    context: &mut HeContext<F>,
) -> Result<Ctxt, Error> {
    let dim = context.encoder.slot_count();
    let dim_half = dim >> 1;
    let n2 = 1 << (dim_half.ilog2() >> 1);
    let n1 = dim_half / n2;

    // Galois keys:
    for index in Bsgs::bsgs_indices(n1, n2, dim) {
        context
            .galois
            .generate_key_for_step(&context.seckey, index)?;
    }
    context.galois.generate_key_for_step(&context.seckey, 0)?;

    // Actual FFT:
    let mat = FFTMatrix::new(dim, root);
    let mut result = ctxt.ctxt_clone()?;
    Bsgs::fully_packed_bsgs(&mut result, &mat, &context.encoder, &context.galois)?;

    Ok(result)
}

fn multiple_packed_fft<F: PrimeField>(
    ctxts: &[Ctxt],
    dim: usize,
    root: F,
    context: &mut HeContext<F>,
) -> Result<Vec<Ctxt>, Error> {
    let slots = context.encoder.slot_count();
    let slots_half = slots >> 1;
    let n2 = 1 << (slots_half.ilog2() >> 1);
    let n1 = slots_half / n2;

    // Galois keys:
    for index in Bsgs::bsgs_indices(n1, n2, slots) {
        context
            .galois
            .generate_key_for_step(&context.seckey, index)?;
    }
    context.galois.generate_key_for_step(&context.seckey, 0)?;

    // Actual FFT:
    let mat = FFTMatrix::new(dim, root);
    Bsgs::bsgs_multiple_of_packsize(ctxts, &mat, &context.encoder, &context.galois)
}

fn fft_selector<F: PrimeField>(
    dim: usize,
    root: F,
    ctxts: &[Ctxt],
    context: &mut HeContext<F>,
) -> Result<Vec<Ctxt>, Error> {
    let result = match dim.cmp(&context.encoder.slot_count()) {
        std::cmp::Ordering::Less => {
            vec![packed_fft(&ctxts[0], dim, root, context)?]
        }
        std::cmp::Ordering::Equal => {
            vec![fully_packed_fft(&ctxts[0], root, context)?]
        }
        std::cmp::Ordering::Greater => multiple_packed_fft(ctxts, dim, root, context)?,
    };
    Ok(result)
}

fn fft_test<F: PrimeField>(dim: usize, context: &mut HeContext<F>) -> Result<(), Error> {
    let mut rng = thread_rng();
    if !dim.is_power_of_two() {
        return Err(Error::Other("FFT: Size must be a power of two".to_string()));
    }

    // let root = FFTMatrix::get_groth16_root(dim);
    let root = FFTMatrix::get_minimal_root(dim);
    let ntt_proc = NTTProcessor::new(dim, root);

    let input = random_vec(dim, &mut rng);
    let expected_output = ntt_proc.fft(&input);

    let ctxts = encrypt(&input, context)?;
    let ctxts_fft = fft_selector(dim, root, &ctxts, context)?;
    let output = decrypt(dim, &ctxts_fft, context)?;

    if output != expected_output {
        return Err(Error::Other("FFT: Results mismatched".to_string()));
    }
    Ok(())
}

fn main() -> color_eyre::Result<ExitCode> {
    let mut context = HeContext::<ark_bn254::Fr>::new(HE_M, HE_BITS)?;

    fft_test(1024, &mut context)?;

    Ok(ExitCode::SUCCESS)
}
