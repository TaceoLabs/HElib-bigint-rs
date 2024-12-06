use helib_rs::{BatchEncoder, CLong, Context, Error, GaloisEngine, PubKey, SecKey, ZZ};
use std::process::ExitCode;

const HE_N: CLong = 1024;
const HE_M: CLong = 2 * HE_N;
const HE_BITS: CLong = 700;

fn main() -> Result<ExitCode, Error> {
    let p = ZZ::char::<ark_bn254::Fr>().unwrap();
    let context = Context::build(HE_M, &p, HE_BITS).unwrap();
    let mut galois = GaloisEngine::build(HE_M as CLong).unwrap();
    let seckey = SecKey::build(&context).unwrap();
    let pubkey = PubKey::from_seckey(&seckey).unwrap();
    // let batch_encoder = BatchEncoder::new(HE_M);

    todo!();

    Ok(ExitCode::SUCCESS)
}
