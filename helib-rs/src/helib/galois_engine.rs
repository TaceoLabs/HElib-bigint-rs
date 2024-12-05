use super::{error::Error, CLong};
use crate::{Ctxt, SecKey};
use std::{ffi::c_void, ptr::null_mut};

#[derive(Debug)]
pub struct GaloisEngine {
    pub(crate) ptr: *mut c_void,
}

impl GaloisEngine {
    pub fn build(m: CLong) -> Result<Self, Error> {
        let mut ptr = null_mut();
        let ret = unsafe { helib_bindings::GK_build(&mut ptr, m) };
        Error::error_from_return(ret)?;
        Ok(Self { ptr })
    }

    pub fn destroy(&mut self) -> Result<(), Error> {
        if self.ptr.is_null() {
            return Ok(());
        }

        let ret = unsafe { helib_bindings::GK_destroy(self.ptr) };
        Error::error_from_return(ret)?;
        self.ptr = null_mut();
        Ok(())
    }

    pub fn generate_key_for_step(&mut self, secky: &SecKey, step: i32) -> Result<(), Error> {
        let ret = unsafe { helib_bindings::GK_generate_step(self.ptr, secky.ptr, step) };
        Error::error_from_return(ret)
    }

    pub fn rotate_ctxt(&self, ctxt: &mut Ctxt, step: i32) -> Result<(), Error> {
        let ret = unsafe { helib_bindings::GK_rotate(self.ptr, ctxt.ptr, step) };
        Error::error_from_return(ret)
    }

    pub fn rotate_ctxt_columns(&self, ctxt: &mut Ctxt) -> Result<(), Error> {
        self.rotate_ctxt(ctxt, 0)
    }
}

impl Drop for GaloisEngine {
    fn drop(&mut self) {
        self.destroy().expect("GaloisEngine destroy failed");
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{encoding::rotate_plain, BatchEncoder, Context, EncodedPtxt, PubKey, ZZ};
    use ark_ff::UniformRand;
    use rand::{thread_rng, Rng};

    const TESTRUNS: usize = 10;
    const N: usize = 16384;
    const M: usize = 2 * N;

    #[test]
    fn build_galois_engine() {
        let mut galois = GaloisEngine::build(16384).unwrap();
        galois.destroy().unwrap(); // Is also called in drop
    }

    #[test]
    fn rotate_test() {
        let batch_encoder = BatchEncoder::new(N);

        let p = ZZ::char::<ark_bn254::Fr>().unwrap();
        let context = Context::build(M as CLong, &p, 700).unwrap();
        let mut galois = GaloisEngine::build(M as CLong).unwrap();
        let seckey = SecKey::build(&context).unwrap();
        let pubkey = PubKey::from_seckey(&seckey).unwrap();

        let mut rng = thread_rng();

        let mut input: Vec<_> = (0..N).map(|_| ark_bn254::Fr::rand(&mut rng)).collect();
        let encoded = EncodedPtxt::encode(&input, &batch_encoder).unwrap();
        let mut ctxt = pubkey.packed_encrypt(&encoded).unwrap();
        for _ in 0..TESTRUNS {
            // rotate rows
            let index = rng.gen_range(1..N >> 1);
            let sign: bool = rng.gen();
            let step = if sign { -(index as i32) } else { index as i32 };

            galois.generate_key_for_step(&seckey, step).unwrap();
            galois.rotate_ctxt(&mut ctxt, step).unwrap();

            let decrypted = seckey.packed_decrypt(&ctxt).unwrap();
            let decoded = decrypted.decode(&batch_encoder).unwrap();
            let expected = rotate_plain(&input, step);
            input = expected;
            assert_eq!(decoded, input);
        }

        // rotate columns
        galois.generate_key_for_step(&seckey, 0).unwrap();
        galois.rotate_ctxt_columns(&mut ctxt).unwrap();

        let decrypted = seckey.packed_decrypt(&ctxt).unwrap();
        let decoded = decrypted.decode(&batch_encoder).unwrap();
        let expected = rotate_plain(&input, 0);
        input = expected;
        assert_eq!(decoded, input);
    }
}
