use ark_ff::PrimeField;
use helib_rs::{
    matrix::{Bsgs, FFTMatrix},
    BatchEncoder, CLong, Context, Ctxt, EncodedPtxt, Error, GaloisEngine, NTTProcessor, PubKey,
    SecKey, ZZ,
};
use rand::{thread_rng, Rng};
use std::{process::ExitCode, time::Instant};

const HE_N: CLong = 32768;
const HE_M: CLong = 2 * HE_N;
const HE_BITS: CLong = 850;

struct HeContext<F: PrimeField> {
    context: Context,
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
            context,
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
    tracing::info!("Encrypting inputs of size: {}", inputs.len());
    let start = Instant::now();

    let mut min_noise_budget = CLong::MAX;
    let mut ctxts = Vec::with_capacity(inputs.len().div_ceil(HE_N as usize));
    for inp in inputs.chunks(HE_N as usize) {
        let encode = EncodedPtxt::encode(inp, &context.encoder)?;
        let ctxt = context.pubkey.packed_encrypt(&encode)?;
        let noise = ctxt.noise_budget()?;
        min_noise_budget = min_noise_budget.min(noise);
        ctxts.push(ctxt);
    }

    let duration_ms = start.elapsed().as_micros() as f64 / 1000.;
    tracing::info!("Encrypting took {} ms", duration_ms);
    tracing::info!("Noise budget: {} bit", min_noise_budget);
    Ok(ctxts)
}

fn decrypt<F: PrimeField>(
    size: usize,
    ctxts: &[Ctxt],
    context: &HeContext<F>,
) -> Result<Vec<F>, Error> {
    tracing::info!("Decrypting outputs of size: {}", size);
    let start = Instant::now();

    let mut min_noise_budget = CLong::MAX;
    let mut outputs = Vec::with_capacity(ctxts.len() * HE_N as usize);
    for ctxt in ctxts {
        let noise = ctxt.noise_budget()?;
        min_noise_budget = min_noise_budget.min(noise);
        let ptxt = context.seckey.packed_decrypt(ctxt)?;
        let output = ptxt.decode(&context.encoder)?;
        outputs.extend(output);
    }
    outputs.resize(size, F::zero());

    let duration_ms = start.elapsed().as_micros() as f64 / 1000.;
    tracing::info!("Decrypting took {} ms", duration_ms);
    tracing::info!("Remaining noise budget was: {} bit", min_noise_budget);
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
    tracing::info!("Adding missing Galois keys");
    let start = Instant::now();
    for index in Bsgs::bsgs_indices(n1, n2, context.encoder.slot_count()) {
        context
            .galois
            .generate_key_for_step(&context.seckey, index)?;
    }
    let duration_ms = start.elapsed().as_micros() as f64 / 1000.;
    tracing::info!("Adding missing Galois keys took {} ms", duration_ms);

    // Actual FFT:
    tracing::info!("Doing FFT in HE");
    let mat = FFTMatrix::new(dim, root);
    let mut result = ctxt.ctxt_clone()?;
    let start = Instant::now();
    Bsgs::babystep_giantstep(&mut result, &mat, &context.encoder, &context.galois, n1, n2)?;
    let duration_ms = start.elapsed().as_micros() as f64 / 1000.;
    tracing::info!("FFT in HE took {} ms", duration_ms);

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
    tracing::info!("Adding missing Galois keys");
    let start = Instant::now();
    for index in Bsgs::bsgs_indices(n1, n2, dim) {
        context
            .galois
            .generate_key_for_step(&context.seckey, index)?;
    }
    context.galois.generate_key_for_step(&context.seckey, 0)?;
    let duration_ms = start.elapsed().as_micros() as f64 / 1000.;
    tracing::info!("Adding missing Galois keys took {} ms", duration_ms);

    // Actual FFT:
    tracing::info!("Doing FFT in HE");
    let mat = FFTMatrix::new(dim, root);
    let mut result = ctxt.ctxt_clone()?;
    let start = Instant::now();
    Bsgs::fully_packed_bsgs(&mut result, &mat, &context.encoder, &context.galois)?;
    let duration_ms = start.elapsed().as_micros() as f64 / 1000.;
    tracing::info!("FFT in HE took {} ms", duration_ms);

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
    tracing::info!("Adding missing Galois keys");
    let start = Instant::now();
    for index in Bsgs::bsgs_indices(n1, n2, slots) {
        context
            .galois
            .generate_key_for_step(&context.seckey, index)?;
    }
    context.galois.generate_key_for_step(&context.seckey, 0)?;
    let duration_ms = start.elapsed().as_micros() as f64 / 1000.;
    tracing::info!("Adding missing Galois keys took {} ms", duration_ms);

    // Actual FFT:
    tracing::info!("Doing FFT in HE");
    let mat = FFTMatrix::new(dim, root);
    let start = Instant::now();
    let result = Bsgs::bsgs_multiple_of_packsize(ctxts, &mat, &context.encoder, &context.galois)?;
    let duration_ms = start.elapsed().as_micros() as f64 / 1000.;
    tracing::info!("FFT in HE took {} ms", duration_ms);

    Ok(result)
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
    tracing::info!("");
    tracing::info!("FFT test for size: {}", dim);

    let mut rng = thread_rng();
    if !dim.is_power_of_two() {
        return Err(Error::Other("FFT: Size must be a power of two".to_string()));
    }
    tracing::info!("Generating random input");
    let start = Instant::now();
    let input = random_vec(dim, &mut rng);
    let duration_ms = start.elapsed().as_micros() as f64 / 1000.;
    tracing::info!("Generating random input took {} ms", duration_ms);

    // let root = FFTMatrix::get_groth16_root(dim);
    tracing::info!("Doing FFT in plain");
    let root = FFTMatrix::get_minimal_root(dim);
    let ntt_proc = NTTProcessor::new(dim, root);
    let start = Instant::now();
    let expected_output = ntt_proc.fft(&input);
    let duration_ms = start.elapsed().as_micros() as f64 / 1000.;
    tracing::info!("FFT in plain took {} ms", duration_ms);

    let ctxts = encrypt(&input, context)?;
    let ctxts_fft = fft_selector(dim, root, &ctxts, context)?;
    let output = decrypt(dim, &ctxts_fft, context)?;

    if output != expected_output {
        return Err(Error::Other("FFT: Results mismatched".to_string()));
    }
    Ok(())
}

fn install_tracing() {
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    let fmt_layer = fmt::layer()
        .with_target(false)
        .with_line_number(false)
        .with_timer(());
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();
}

fn main() -> color_eyre::Result<ExitCode> {
    install_tracing();

    let mut context = HeContext::<ark_bn254::Fr>::new(HE_M, HE_BITS)?;
    let security = context.context.security_level()?;
    tracing::info!("HE Parameters:");
    tracing::info!("  N: {}", HE_N);
    tracing::info!("  Bits: {}", HE_BITS);
    tracing::info!("  Security level: {:.2}", security);

    let ffts_bit = [8, 9, 10, 11, 12, 13, 14, 15, 16, 17];

    for bit in ffts_bit {
        fft_test(1 << bit, &mut context)?;
    }

    Ok(ExitCode::SUCCESS)
}
