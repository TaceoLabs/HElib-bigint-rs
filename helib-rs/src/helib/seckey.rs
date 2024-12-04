use super::{ctxt::Ctxt, error::Error, pubkey::PubKey};
use crate::{Context, ZZ};
use std::{ffi::c_void, ptr::null_mut};

#[derive(Debug)]
pub struct SecKey {
    pub(crate) ptr: *mut c_void,
}

impl SecKey {
    pub fn build(context: &Context) -> Result<Self, Error> {
        let mut ptr = null_mut();
        let ret = unsafe { helib_bindings::seckey_build(&mut ptr, context.ptr) };
        Error::error_from_return(ret)?;
        Ok(Self { ptr })
    }

    pub fn destroy(&mut self) -> Result<(), Error> {
        if self.ptr.is_null() {
            return Ok(());
        }

        let ret = unsafe { helib_bindings::seckey_destroy(self.ptr) };
        Error::error_from_return(ret)?;
        self.ptr = null_mut();
        Ok(())
    }

    pub fn get_public_key(&self) -> Result<PubKey, Error> {
        PubKey::from_seckey(self)
    }

    pub fn encrypt(&self, zz: &ZZ) -> Result<Ctxt, Error> {
        let mut ctxt = Ctxt::empty_pointer();
        let ret = unsafe { helib_bindings::seckey_encrypt(&mut ctxt.ptr, self.ptr, zz.ptr) };
        Error::error_from_return(ret)?;
        Ok(ctxt)
    }

    pub fn decrypt(&self, ctxt: &Ctxt) -> Result<ZZ, Error> {
        let mut zz = ZZ::empty_pointer();
        let ret = unsafe { helib_bindings::seckey_decrypt(&mut zz.ptr, self.ptr, ctxt.ptr) };
        Error::error_from_return(ret)?;
        Ok(zz)
    }
}

impl Drop for SecKey {
    fn drop(&mut self) {
        self.destroy().expect("SecKey destroy failed");
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ZZ;
    use ark_ff::UniformRand;
    use rand::thread_rng;

    const TESTRUNS: usize = 10;

    #[test]
    fn build_seckey() {
        let p = ZZ::char::<ark_bn254::Fr>().unwrap();
        let context = Context::build(32109, &p, 700).unwrap();
        let mut seckey = SecKey::build(&context).unwrap();
        seckey.destroy().unwrap(); // Is also called in drop
    }

    #[test]
    fn seckey_get_pubkey() {
        let p = ZZ::char::<ark_bn254::Fr>().unwrap();
        let context = Context::build(32109, &p, 700).unwrap();
        let seckey = SecKey::build(&context).unwrap();
        let _pubkey = seckey.get_public_key().unwrap();
    }

    #[test]
    fn seckey_encrypt_decrypt() {
        let p = ZZ::char::<ark_bn254::Fr>().unwrap();
        let context = Context::build(32109, &p, 700).unwrap();
        let seckey = SecKey::build(&context).unwrap();
        let mut rng = thread_rng();
        for _ in 0..TESTRUNS {
            let input = ark_bn254::Fr::rand(&mut rng);
            let zz = ZZ::from_primefield(input).unwrap();
            let ctxt = seckey.encrypt(&zz).unwrap();
            let ptxt = seckey.decrypt(&ctxt).unwrap();
            let decrypted = ptxt.to_primefield::<ark_bn254::Fr>().unwrap();
            assert_eq!(decrypted, input);
        }
    }
}
