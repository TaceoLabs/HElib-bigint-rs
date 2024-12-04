use super::error::Error;
use std::{ffi::c_void, ptr::null_mut};

#[derive(Debug)]
pub struct Ctxt {
    pub(crate) ptr: *mut c_void,
}

impl Ctxt {
    pub(crate) fn empty_pointer() -> Self {
        Self { ptr: null_mut() }
    }

    pub fn destroy(&mut self) -> Result<(), Error> {
        if self.ptr.is_null() {
            return Ok(());
        }

        let ret = unsafe { helib_bindings::ctxt_destroy(self.ptr) };
        Error::error_from_return(ret)?;
        self.ptr = null_mut();
        Ok(())
    }
}

impl Drop for Ctxt {
    fn drop(&mut self) {
        self.destroy().expect("Ctxt destroy failed");
    }
}

#[cfg(test)]
mod test {
    use crate::{
        helib::{pubkey::PubKey, seckey::SecKey},
        Context, ZZ,
    };
    use ark_ff::UniformRand;
    use rand::thread_rng;

    #[test]
    fn build_ctxt() {
        let p = ZZ::char::<ark_bn254::Fr>().unwrap();
        let context = Context::build(32109, &p, 700).unwrap();
        let seckey = SecKey::build(&context).unwrap();
        let pubkey = PubKey::from_seckey(&seckey).unwrap();
        let mut rng = thread_rng();
        let zz = ZZ::from_primefield(ark_bn254::Fr::rand(&mut rng)).unwrap();
        let mut ctxt = pubkey.encrypt(&zz).unwrap();
        ctxt.destroy().unwrap(); // Is also called in drop
    }
}
