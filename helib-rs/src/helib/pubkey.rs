use crate::ZZ;

use super::{ctxt::Ctxt, error::Error, seckey::SecKey};
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
}

impl Drop for PubKey {
    fn drop(&mut self) {
        self.destroy().expect("PubKey destroy failed");
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Context, ZZ};
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
            let zz = ZZ::from_primefield(input).unwrap();
            let ctxt = pubkey.encrypt(&zz).unwrap();
            let ptxt = seckey.decrypt(&ctxt).unwrap();
            let decrypted = ptxt.to_primefield::<ark_bn254::Fr>().unwrap();
            assert_eq!(decrypted, input);
        }
    }
}
