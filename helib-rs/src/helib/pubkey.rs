use super::{ctxt::Ctxt, error::Error, seckey::SecKey};
use crate::{EncodedPtxt, ZZ};
use ark_ff::PrimeField;
use std::{ffi::c_void, ptr::null_mut};

#[derive(Debug)]
pub struct PubKey {
    pub(crate) ptr: *mut c_void,
}

impl PubKey {
    pub fn from_seckey(seckey: &SecKey) -> Result<Self, Error> {
        let mut ptr = null_mut();
        let ret = unsafe { helib_bindings::pubkey_from_seckey(&mut ptr, seckey.ptr) };
        Error::error_from_return(ret)?;
        Ok(Self { ptr })
    }

    pub fn destroy(&mut self) -> Result<(), Error> {
        if self.ptr.is_null() {
            return Ok(());
        }

        let ret = unsafe { helib_bindings::pubkey_destroy(self.ptr) };
        Error::error_from_return(ret)?;
        self.ptr = null_mut();
        Ok(())
    }

    pub fn encrypt(&self, zz: &ZZ) -> Result<Ctxt, Error> {
        let mut ctxt = Ctxt::empty_pointer();
        let ret = unsafe { helib_bindings::pubkey_encrypt(&mut ctxt.ptr, self.ptr, zz.ptr) };
        Error::error_from_return(ret)?;
        Ok(ctxt)
    }

    pub fn packed_encrypt(&self, ptxt: &EncodedPtxt) -> Result<Ctxt, Error> {
        let mut ctxt = Ctxt::empty_pointer();
        let ret =
            unsafe { helib_bindings::pubkey_packed_encrypt(&mut ctxt.ptr, self.ptr, ptxt.ptr) };
        Error::error_from_return(ret)?;
        Ok(ctxt)
    }

    pub fn encrypt_fieldelement<F: PrimeField>(&self, field: F) -> Result<Ctxt, Error> {
        let zz = ZZ::from_fieldelement(field)?;
        self.encrypt(&zz)
    }
}

impl Drop for PubKey {
    fn drop(&mut self) {
        self.destroy().expect("PubKey destroy failed");
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{helib::CLong, BatchEncoder, Context, ZZ};
    use ark_ff::UniformRand;
    use rand::thread_rng;

    const TESTRUNS: usize = 10;

    #[test]
    fn build_pubkey_from_seckey() {
        let p = ZZ::char::<ark_bn254::Fr>().unwrap();
        let context = Context::build(32109, &p, 700).unwrap();
        let seckey = SecKey::build(&context).unwrap();
        let mut pubkey = PubKey::from_seckey(&seckey).unwrap();
        pubkey.destroy().unwrap(); // Is also called in drop
    }

    #[test]
    fn pubkey_encrypt_decrypt() {
        let p = ZZ::char::<ark_bn254::Fr>().unwrap();
        let context = Context::build(32109, &p, 700).unwrap();
        let seckey = SecKey::build(&context).unwrap();
        let pubkey = PubKey::from_seckey(&seckey).unwrap();
        let mut rng = thread_rng();
        for _ in 0..TESTRUNS {
            let input = ark_bn254::Fr::rand(&mut rng);
            let zz = ZZ::from_fieldelement(input).unwrap();
            let ctxt = pubkey.encrypt(&zz).unwrap();
            let ptxt = seckey.decrypt(&ctxt).unwrap();
            let decrypted = ptxt.to_fieldelement::<ark_bn254::Fr>().unwrap();
            assert_eq!(decrypted, input);
        }
    }

    #[test]
    fn pubkey_encrypt_decrypt_fieldelemen() {
        let p = ZZ::char::<ark_bn254::Fr>().unwrap();
        let context = Context::build(32109, &p, 700).unwrap();
        let seckey = SecKey::build(&context).unwrap();
        let pubkey = PubKey::from_seckey(&seckey).unwrap();
        let mut rng = thread_rng();
        for _ in 0..TESTRUNS {
            let input = ark_bn254::Fr::rand(&mut rng);
            let ctxt = pubkey.encrypt_fieldelement(input).unwrap();
            let decrypted = seckey.decrypt_fieldelement::<ark_bn254::Fr>(&ctxt).unwrap();
            assert_eq!(decrypted, input);
        }
    }

    #[test]
    fn pubkey_packed_encrypt_decrypt() {
        const N: usize = 16384;
        const M: usize = 2 * N;
        let batch_encoder = BatchEncoder::new(M as CLong);

        let p = ZZ::char::<ark_bn254::Fr>().unwrap();
        let context = Context::build(M as CLong, &p, 700).unwrap();
        let seckey = SecKey::build(&context).unwrap();
        let pubkey = PubKey::from_seckey(&seckey).unwrap();

        let mut rng = thread_rng();
        for _ in 0..TESTRUNS {
            let input: Vec<_> = (0..N).map(|_| ark_bn254::Fr::rand(&mut rng)).collect();
            let ptxt = EncodedPtxt::encode(&input, &batch_encoder).unwrap();
            let ctxt = pubkey.packed_encrypt(&ptxt).unwrap();
            let ptxt_ = seckey.packed_decrypt(&ctxt).unwrap();
            let output = ptxt_.decode(&batch_encoder).unwrap();
            assert_eq!(input, output);
        }
    }
}
