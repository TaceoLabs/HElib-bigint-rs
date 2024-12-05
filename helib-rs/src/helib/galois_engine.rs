use super::{error::Error, CLong};
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
}

impl Drop for GaloisEngine {
    fn drop(&mut self) {
        self.destroy().expect("GaloisEngine destroy failed");
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn build_galois_engine() {
        let mut galois = GaloisEngine::build(16384).unwrap();
        galois.destroy().unwrap(); // Is also called in drop
    }
}
