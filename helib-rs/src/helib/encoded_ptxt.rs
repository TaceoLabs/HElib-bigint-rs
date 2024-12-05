use super::{error::Error, CLong};
use crate::{BatchEncoder, ZZ};
use ark_ff::PrimeField;
use std::{ffi::c_void, ptr::null_mut};

#[derive(Debug)]
pub struct EncodedPtxt {
    pub(crate) ptr: *mut c_void,
}

impl EncodedPtxt {
    pub(crate) fn empty_pointer() -> Self {
        Self { ptr: null_mut() }
    }

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
        if read.len() != batch_encoder.slot_count() {
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
    use crate::{Context, PubKey, SecKey};
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

    #[test]
    fn packed_arithmetic_test() {
        const N: usize = 16384;
        const M: usize = 2 * N;

        let batch_encoder = BatchEncoder::new(N);

        let p = ZZ::char::<ark_bn254::Fr>().unwrap();
        let context = Context::build(M as CLong, &p, 700).unwrap();
        let seckey = SecKey::build(&context).unwrap();
        let pubkey = PubKey::from_seckey(&seckey).unwrap();

        let mut rng = thread_rng();
        for _ in 0..TESTRUNS {
            let a: Vec<_> = (0..N).map(|_| ark_bn254::Fr::rand(&mut rng)).collect();
            let b: Vec<_> = (0..N).map(|_| ark_bn254::Fr::rand(&mut rng)).collect();
            let enc_a = EncodedPtxt::encode(&a, &batch_encoder).unwrap();
            let enc_b = EncodedPtxt::encode(&b, &batch_encoder).unwrap();
            let ctxt_a = pubkey.packed_encrypt(&enc_a).unwrap();
            let ctxt_b = pubkey.packed_encrypt(&enc_b).unwrap();

            let ctxt_add = &ctxt_a + &ctxt_b;
            let ctxt_sub = &ctxt_a - &ctxt_b;
            let ctxt_neg = -&ctxt_a;
            let ctxt_mul = &ctxt_a * &ctxt_b;

            let enc_add = seckey.packed_decrypt(&ctxt_add).unwrap();
            let enc_sub = seckey.packed_decrypt(&ctxt_sub).unwrap();
            let enc_neg = seckey.packed_decrypt(&ctxt_neg).unwrap();
            let enc_mul = seckey.packed_decrypt(&ctxt_mul).unwrap();

            let add = enc_add.decode(&batch_encoder).unwrap();
            let sub = enc_sub.decode(&batch_encoder).unwrap();
            let neg = enc_neg.decode(&batch_encoder).unwrap();
            let mul = enc_mul.decode(&batch_encoder).unwrap();

            assert_eq!(
                add,
                a.iter()
                    .zip(b.iter())
                    .map(|(a, b)| a + b)
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                sub,
                a.iter()
                    .zip(b.iter())
                    .map(|(a, b)| a - b)
                    .collect::<Vec<_>>()
            );
            assert_eq!(neg, a.iter().map(|a| -*a).collect::<Vec<_>>());
            assert_eq!(
                mul,
                a.iter()
                    .zip(b.iter())
                    .map(|(a, b)| a * b)
                    .collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn packed_arithmetic_inplace_test() {
        const N: usize = 16384;
        const M: usize = 2 * N;

        let batch_encoder = BatchEncoder::new(N);

        let p = ZZ::char::<ark_bn254::Fr>().unwrap();
        let context = Context::build(M as CLong, &p, 700).unwrap();
        let seckey = SecKey::build(&context).unwrap();
        let pubkey = PubKey::from_seckey(&seckey).unwrap();

        let mut rng = thread_rng();
        for _ in 0..TESTRUNS {
            let a: Vec<_> = (0..N).map(|_| ark_bn254::Fr::rand(&mut rng)).collect();
            let b: Vec<_> = (0..N).map(|_| ark_bn254::Fr::rand(&mut rng)).collect();
            let enc_a = EncodedPtxt::encode(&a, &batch_encoder).unwrap();
            let enc_b = EncodedPtxt::encode(&b, &batch_encoder).unwrap();
            let ctxt_a = pubkey.packed_encrypt(&enc_a).unwrap();
            let ctxt_b = pubkey.packed_encrypt(&enc_b).unwrap();

            let mut ctxt_add = ctxt_a.clone();
            ctxt_add += &ctxt_b;
            let mut ctxt_sub = ctxt_a.clone();
            ctxt_sub -= &ctxt_b;
            let mut ctxt_neg = ctxt_a.clone();
            ctxt_neg.negate_inplace().unwrap();
            let mut ctxt_mul = ctxt_a.clone();
            ctxt_mul *= &ctxt_b;

            let enc_add = seckey.packed_decrypt(&ctxt_add).unwrap();
            let enc_sub = seckey.packed_decrypt(&ctxt_sub).unwrap();
            let enc_neg = seckey.packed_decrypt(&ctxt_neg).unwrap();
            let enc_mul = seckey.packed_decrypt(&ctxt_mul).unwrap();

            let add = enc_add.decode(&batch_encoder).unwrap();
            let sub = enc_sub.decode(&batch_encoder).unwrap();
            let neg = enc_neg.decode(&batch_encoder).unwrap();
            let mul = enc_mul.decode(&batch_encoder).unwrap();

            assert_eq!(
                add,
                a.iter()
                    .zip(b.iter())
                    .map(|(a, b)| a + b)
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                sub,
                a.iter()
                    .zip(b.iter())
                    .map(|(a, b)| a - b)
                    .collect::<Vec<_>>()
            );
            assert_eq!(neg, a.iter().map(|a| -*a).collect::<Vec<_>>());
            assert_eq!(
                mul,
                a.iter()
                    .zip(b.iter())
                    .map(|(a, b)| a * b)
                    .collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn packed_arithmetic_with_const_test() {
        const N: usize = 16384;
        const M: usize = 2 * N;

        let batch_encoder = BatchEncoder::new(N);

        let p = ZZ::char::<ark_bn254::Fr>().unwrap();
        let context = Context::build(M as CLong, &p, 700).unwrap();
        let seckey = SecKey::build(&context).unwrap();
        let pubkey = PubKey::from_seckey(&seckey).unwrap();

        let mut rng = thread_rng();
        for _ in 0..TESTRUNS {
            let a: Vec<_> = (0..N).map(|_| ark_bn254::Fr::rand(&mut rng)).collect();
            let b: Vec<_> = (0..N).map(|_| ark_bn254::Fr::rand(&mut rng)).collect();
            let enc_a = EncodedPtxt::encode(&a, &batch_encoder).unwrap();
            let enc_b = EncodedPtxt::encode(&b, &batch_encoder).unwrap();
            let ctxt_a = pubkey.packed_encrypt(&enc_a).unwrap();

            let ctxt_add = &ctxt_a + &enc_b;
            let ctxt_sub = &ctxt_a - &enc_b;
            let ctxt_sub2 = &enc_b - &ctxt_a;
            let ctxt_mul = &ctxt_a * &enc_b;

            let enc_add = seckey.packed_decrypt(&ctxt_add).unwrap();
            let enc_sub = seckey.packed_decrypt(&ctxt_sub).unwrap();
            let enc_sub2 = seckey.packed_decrypt(&ctxt_sub2).unwrap();
            let enc_mul = seckey.packed_decrypt(&ctxt_mul).unwrap();

            let add = enc_add.decode(&batch_encoder).unwrap();
            let sub = enc_sub.decode(&batch_encoder).unwrap();
            let sub2 = enc_sub2.decode(&batch_encoder).unwrap();
            let mul = enc_mul.decode(&batch_encoder).unwrap();

            assert_eq!(
                add,
                a.iter()
                    .zip(b.iter())
                    .map(|(a, b)| a + b)
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                sub,
                a.iter()
                    .zip(b.iter())
                    .map(|(a, b)| a - b)
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                sub2,
                a.iter()
                    .zip(b.iter())
                    .map(|(a, b)| b - a)
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                mul,
                a.iter()
                    .zip(b.iter())
                    .map(|(a, b)| a * b)
                    .collect::<Vec<_>>()
            );
        }
    }
}
