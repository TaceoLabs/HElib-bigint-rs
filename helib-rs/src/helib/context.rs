use std::{ffi::c_void, ptr::null_mut};

#[derive(Debug)]
pub struct Context {
    pub(crate) ptr: *mut c_void,
}
