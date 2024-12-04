use super::{error::Error, CLong};
use crate::{BatchEncoder, ZZ};
use ark_ff::PrimeField;
use std::{ffi::c_void, ptr::null_mut};

#[derive(Debug)]
pub struct EncodedPtxt {
    pub(crate) ptr: *mut c_void,
}

impl EncodedPtxt {
    pub(crate) fn from_len(len: usize) -> Result<Self, Error> {
        let mut ptr = null_mut();
        let ret = unsafe { helib_bindings::ZZX_from_len(&mut ptr, len as CLong) };
        Error::error_from_return(ret)?;
        Ok(Self { ptr })
    }

    pub(crate) fn set_index(&mut self, index: usize, value: &ZZ) -> Result<(), Error> {
        let ret = unsafe { helib_bindings::ZZX_set_index(self.ptr, index as CLong, value.ptr) };
        Error::error_from_return(ret)
    }

    pub(crate) fn get_index(&self, index: usize) -> Result<ZZ, Error> {
        let mut zz = ZZ::empty_pointer();
        let ret = unsafe { helib_bindings::ZZX_get_index(&mut zz.ptr, self.ptr, index as CLong) };
        Error::error_from_return(ret)?;
        Ok(zz)
    }

    pub(crate) fn get_len(&self) -> Result<usize, Error> {
        let mut len = 0;
        let ret = unsafe { helib_bindings::ZZX_get_length(self.ptr, &mut len) };
        Error::error_from_return(ret)?;
        Ok(len as usize)
    }

    pub fn encode<F: PrimeField>(
        vec: &[F],
        batch_encoder: &BatchEncoder<F>,
    ) -> Result<Self, Error> {
        if vec.len() > batch_encoder.slot_count() {
            return Err(Error::BatchSlots);
        }
        let encoded = batch_encoder.encode(vec);

        let mut encoded_ptxt = EncodedPtxt::from_len(encoded.len())?;
        for (i, val) in encoded.into_iter().enumerate() {
            let zz = ZZ::from_fieldelement(val)?;
            encoded_ptxt.set_index(i, &zz)?;
        }
        Ok(encoded_ptxt)
    }

    pub fn decode<F: PrimeField>(&self, batch_encoder: &BatchEncoder<F>) -> Result<Vec<F>, Error> {
        let len = self.get_len()?;
        let mut read = Vec::with_capacity(len);
        for i in 0..len {
            let zz = self.get_index(i)?;
            let val = zz.to_fieldelement()?;
            read.push(val);
        }
        if read.len() > batch_encoder.slot_count() {
            return Err(Error::BatchSlots);
        }
        Ok(batch_encoder.decode(&read))
    }

    pub fn destroy(&mut self) -> Result<(), Error> {
        if self.ptr.is_null() {
            return Ok(());
        }

        let ret = unsafe { helib_bindings::ZZX_destroy(self.ptr) };
        Error::error_from_return(ret)?;
        self.ptr = null_mut();
        Ok(())
    }
}

impl Drop for EncodedPtxt {
    fn drop(&mut self) {
        self.destroy().expect("EncodedPtxt destroy failed");
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ark_ff::UniformRand;
    use rand::thread_rng;

    const TESTRUNS: usize = 10;
    const N: usize = 1024;

    #[test]
    fn encoded_ptxt_create() {
        let mut ptxt = EncodedPtxt::from_len(N).unwrap();
        assert_eq!(ptxt.get_len().unwrap(), N);
        ptxt.destroy().unwrap(); // Is also called in drop
    }

    #[test]
    fn encoded_ptxt_encode_decode_test() {
        let batch_encoder = BatchEncoder::new(N);

        let mut rng = thread_rng();
        for _ in 0..TESTRUNS {
            let input: Vec<_> = (0..N).map(|_| ark_bn254::Fr::rand(&mut rng)).collect();
            let ptxt = EncodedPtxt::encode(&input, &batch_encoder).unwrap();
            let output = ptxt.decode(&batch_encoder).unwrap();
            assert_eq!(input, output);
        }
    }
}
