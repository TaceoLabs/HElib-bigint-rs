use super::{error::Error, seckey::SecKey};
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

    #[test]
    fn build_pubkey_from_seckey() {
        let p = ZZ::char::<ark_bn254::Fr>().unwrap();
        let context = Context::build(32109, &p, 700).unwrap();
        let seckey = SecKey::build(&context).unwrap();
        let mut pubkey = PubKey::from_seckey(&seckey).unwrap();
        pubkey.destroy().unwrap(); // Is also called in drop
    }
}
