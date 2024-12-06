use ark_ff::PrimeField;
use color_eyre::eyre::Error;
use helib_rs::{
    matrix::FFTMatrix, BatchEncoder, CLong, Context, Ctxt, EncodedPtxt, GaloisEngine, NTTProcessor,
    PubKey, SecKey, ZZ,
};
use rand::{thread_rng, Rng};
use std::process::ExitCode;

const HE_N: CLong = 1024;
const HE_M: CLong = 2 * HE_N;
const HE_BITS: CLong = 700;

struct HeContext<F: PrimeField> {
    context: Context,
    seckey: SecKey,
    pubkey: PubKey,
    encoder: BatchEncoder<F>,
    galois: GaloisEngine,
}

impl<F: PrimeField> HeContext<F> {
    fn new(m: CLong, bits: CLong) -> Self {
        let p = ZZ::char::<ark_bn254::Fr>().unwrap();
        let context = Context::build(m, &p, bits).unwrap();
        let galois = GaloisEngine::build(m).unwrap();
        let seckey = SecKey::build(&context).unwrap();
        let pubkey = PubKey::from_seckey(&seckey).unwrap();
        let encoder = BatchEncoder::new(m);

        Self {
            context,
            seckey,
            pubkey,
            encoder,
            galois,
        }
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

fn fft_test<F: PrimeField>(size: usize, context: &mut HeContext<F>) -> Result<(), Error> {
    let mut rng = thread_rng();
    if !size.is_power_of_two() {
        return Err(Error::msg("FFT: Size must be a power of two"));
    }

    // let root = FFTMatrix::get_groth16_root(size);
    let root = FFTMatrix::get_minimal_root(size);
    let ntt_proc = NTTProcessor::new(size, root);

    let input = random_vec(size, &mut rng);
    let expected_output = ntt_proc.fft(&input);
    todo!()
}

fn main() -> color_eyre::Result<ExitCode> {
    let mut context = HeContext::<ark_bn254::Fr>::new(HE_M, HE_BITS);

    todo!();

    Ok(ExitCode::SUCCESS)
}
