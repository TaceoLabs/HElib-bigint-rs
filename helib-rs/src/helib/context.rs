use super::{error::Error, CLong};
use crate::ZZ;
use std::{ffi::c_void, ptr::null_mut};

#[derive(Debug)]
pub struct Context {
    pub(crate) ptr: *mut c_void,
}

impl Context {
    pub fn build(m: CLong, p: &ZZ, bits: CLong) -> Result<Self, Error> {
        let mut ptr = null_mut();
        let ret = unsafe { helib_bindings::context_build(&mut ptr, m, p.ptr, bits) };
        Error::error_from_return(ret)?;
        Ok(Self { ptr })
    }

    pub fn security_level(&self) -> Result<f64, Error> {
        let mut res = 0f64;
        let ret = unsafe { helib_bindings::context_get_security_level(self.ptr, &mut res) };
        Error::error_from_return(ret)?;
        Ok(res)
    }

    pub fn destroy(&mut self) -> Result<(), Error> {
        if self.ptr.is_null() {
            return Ok(());
        }

        let ret = unsafe { helib_bindings::context_destroy(self.ptr) };
        Error::error_from_return(ret)?;
        self.ptr = null_mut();
        Ok(())
    }

    pub fn printout(&self) -> Result<(), Error> {
        let ret = unsafe { helib_bindings::context_printout(self.ptr) };
        Error::error_from_return(ret)
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        self.destroy().expect("Context destroy failed");
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn build_context() {
        let p = ZZ::char::<ark_bn254::Fr>().unwrap();
        let mut context = Context::build(32109, &p, 700).unwrap();
        context.destroy().unwrap(); // Is also called in drop
    }

    #[test]
    fn context_get_security_level() {
        let p = ZZ::char::<ark_bn254::Fr>().unwrap();
        let context = Context::build(32109, &p, 700).unwrap();
        let _level = context.security_level().unwrap();
    }

    #[test]
    #[ignore]
    fn print_context() {
        let p = ZZ::char::<ark_bn254::Fr>().unwrap();
        let context = Context::build(32109, &p, 700).unwrap();
        context.printout().unwrap();
    }
}
