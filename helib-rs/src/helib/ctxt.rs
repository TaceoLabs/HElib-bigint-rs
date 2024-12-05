use super::{error::Error, CLong};
use crate::{EncodedPtxt, ZZ};
use ark_ff::PrimeField;
use std::{
    ffi::c_void,
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
    ptr::null_mut,
};

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

    pub fn noise_budget(&self) -> Result<CLong, Error> {
        let mut res = 0;
        let ret = unsafe { helib_bindings::ctxt_get_noise_budget(self.ptr, &mut res) };
        Error::error_from_return(ret)?;
        Ok(res)
    }

    pub fn ctxt_clone(&self) -> Result<Ctxt, Error> {
        let mut ctxt = Ctxt::empty_pointer();
        let ret = unsafe { helib_bindings::ctxt_clone(&mut ctxt.ptr, self.ptr) };
        Error::error_from_return(ret)?;
        Ok(ctxt)
    }

    // Arithmetic

    pub fn ctxt_add(&self, other: &Ctxt) -> Result<Ctxt, Error> {
        let mut ctxt = Ctxt::empty_pointer();
        let ret = unsafe { helib_bindings::ctxt_add(&mut ctxt.ptr, self.ptr, other.ptr) };
        Error::error_from_return(ret)?;
        Ok(ctxt)
    }

    pub fn ctxt_sub(&self, other: &Ctxt) -> Result<Ctxt, Error> {
        let mut ctxt = Ctxt::empty_pointer();
        let ret = unsafe { helib_bindings::ctxt_sub(&mut ctxt.ptr, self.ptr, other.ptr) };
        Error::error_from_return(ret)?;
        Ok(ctxt)
    }

    pub fn ctxt_negate(&self) -> Result<Ctxt, Error> {
        let mut ctxt = Ctxt::empty_pointer();
        let ret = unsafe { helib_bindings::ctxt_negate(&mut ctxt.ptr, self.ptr) };
        Error::error_from_return(ret)?;
        Ok(ctxt)
    }

    pub fn ctxt_mul(&self, other: &Ctxt) -> Result<Ctxt, Error> {
        let mut ctxt = Ctxt::empty_pointer();
        let ret = unsafe { helib_bindings::ctxt_mult(&mut ctxt.ptr, self.ptr, other.ptr) };
        Error::error_from_return(ret)?;
        Ok(ctxt)
    }

    // Arithmetic in place

    pub fn ctxt_add_inplace(&mut self, other: &Ctxt) -> Result<(), Error> {
        let ret = unsafe { helib_bindings::ctxt_add_inplace(self.ptr, other.ptr) };
        Error::error_from_return(ret)
    }

    pub fn ctxt_sub_inplace(&mut self, other: &Ctxt) -> Result<(), Error> {
        let ret = unsafe { helib_bindings::ctxt_sub_inplace(self.ptr, other.ptr) };
        Error::error_from_return(ret)
    }

    pub fn ctxt_negate_inplace(&mut self) -> Result<(), Error> {
        let ret = unsafe { helib_bindings::ctxt_negate_inplace(self.ptr) };
        Error::error_from_return(ret)
    }

    #[inline(always)]
    pub fn negate_inplace(&mut self) -> Result<(), Error> {
        self.ctxt_negate_inplace()
    }

    pub fn ctxt_mul_inplace(&mut self, other: &Ctxt) -> Result<(), Error> {
        let ret = unsafe { helib_bindings::ctxt_mult_inplace(self.ptr, other.ptr) };
        Error::error_from_return(ret)
    }

    // Arithmetic with constants

    pub fn ctxt_add_by_constant(&self, other: &ZZ) -> Result<Ctxt, Error> {
        let mut ctxt = Ctxt::empty_pointer();
        let ret =
            unsafe { helib_bindings::ctxt_add_by_constant(&mut ctxt.ptr, self.ptr, other.ptr) };
        Error::error_from_return(ret)?;
        Ok(ctxt)
    }

    pub fn ctxt_sub_by_constant(&self, other: &ZZ) -> Result<Ctxt, Error> {
        let mut ctxt = Ctxt::empty_pointer();
        let ret =
            unsafe { helib_bindings::ctxt_sub_by_constant(&mut ctxt.ptr, self.ptr, other.ptr) };
        Error::error_from_return(ret)?;
        Ok(ctxt)
    }

    pub fn ctxt_sub_from_constant(&self, other: &ZZ) -> Result<Ctxt, Error> {
        let mut ctxt = Ctxt::empty_pointer();
        let ret =
            unsafe { helib_bindings::ctxt_sub_from_constant(&mut ctxt.ptr, other.ptr, self.ptr) };
        Error::error_from_return(ret)?;
        Ok(ctxt)
    }

    pub fn ctxt_mul_by_constant(&self, other: &ZZ) -> Result<Ctxt, Error> {
        let mut ctxt = Ctxt::empty_pointer();
        let ret =
            unsafe { helib_bindings::ctxt_mult_by_constant(&mut ctxt.ptr, self.ptr, other.ptr) };
        Error::error_from_return(ret)?;
        Ok(ctxt)
    }

    // Arithmetic with packed constants

    pub fn ctxt_add_by_packed_constant(&self, other: &EncodedPtxt) -> Result<Ctxt, Error> {
        let mut ctxt = Ctxt::empty_pointer();
        let ret = unsafe {
            helib_bindings::ctxt_add_by_packed_constant(&mut ctxt.ptr, self.ptr, other.ptr)
        };
        Error::error_from_return(ret)?;
        Ok(ctxt)
    }

    pub fn ctxt_sub_by_packed_constant(&self, other: &EncodedPtxt) -> Result<Ctxt, Error> {
        let mut ctxt = Ctxt::empty_pointer();
        let ret = unsafe {
            helib_bindings::ctxt_sub_by_packed_constant(&mut ctxt.ptr, self.ptr, other.ptr)
        };
        Error::error_from_return(ret)?;
        Ok(ctxt)
    }

    pub fn ctxt_sub_from_packed_constant(&self, other: &EncodedPtxt) -> Result<Ctxt, Error> {
        let mut ctxt = Ctxt::empty_pointer();
        let ret = unsafe {
            helib_bindings::ctxt_sub_from_packed_constant(&mut ctxt.ptr, other.ptr, self.ptr)
        };
        Error::error_from_return(ret)?;
        Ok(ctxt)
    }

    pub fn ctxt_mul_by_packed_constant(&self, other: &EncodedPtxt) -> Result<Ctxt, Error> {
        let mut ctxt = Ctxt::empty_pointer();
        let ret = unsafe {
            helib_bindings::ctxt_mult_by_packed_constant(&mut ctxt.ptr, self.ptr, other.ptr)
        };
        Error::error_from_return(ret)?;
        Ok(ctxt)
    }

    // Arithmetic with constants in place

    pub fn ctxt_add_by_constant_inplace(&mut self, other: &ZZ) -> Result<(), Error> {
        let ret = unsafe { helib_bindings::ctxt_add_by_constant_inplace(self.ptr, other.ptr) };
        Error::error_from_return(ret)
    }

    pub fn ctxt_sub_by_constant_inplace(&mut self, other: &ZZ) -> Result<(), Error> {
        let ret = unsafe { helib_bindings::ctxt_sub_by_constant_inplace(self.ptr, other.ptr) };
        Error::error_from_return(ret)
    }

    pub fn ctxt_sub_from_constant_inplace(&mut self, other: &ZZ) -> Result<(), Error> {
        let ret = unsafe { helib_bindings::ctxt_sub_from_constant_inplace(self.ptr, other.ptr) };
        Error::error_from_return(ret)
    }

    pub fn ctxt_mul_by_constant_inplace(&mut self, other: &ZZ) -> Result<(), Error> {
        let ret = unsafe { helib_bindings::ctxt_mult_by_constant_inplace(self.ptr, other.ptr) };
        Error::error_from_return(ret)
    }

    // Arithmetic with primefield elements

    pub fn ctxt_add_by_field_element<F: PrimeField>(&self, other: F) -> Result<Ctxt, Error> {
        let zz = ZZ::from_fieldelement(other)?;
        self.ctxt_add_by_constant(&zz)
    }

    pub fn ctxt_sub_by_field_element<F: PrimeField>(&self, other: F) -> Result<Ctxt, Error> {
        let zz = ZZ::from_fieldelement(other)?;
        self.ctxt_sub_by_constant(&zz)
    }

    pub fn ctxt_sub_from_field_element<F: PrimeField>(&self, other: F) -> Result<Ctxt, Error> {
        let zz = ZZ::from_fieldelement(other)?;
        self.ctxt_sub_from_constant(&zz)
    }

    pub fn ctxt_mul_by_field_element<F: PrimeField>(&self, other: F) -> Result<Ctxt, Error> {
        let zz = ZZ::from_fieldelement(other)?;
        self.ctxt_mul_by_constant(&zz)
    }

    // Arithmetic with primefield elements in place

    pub fn ctxt_add_by_field_element_inplace<F: PrimeField>(
        &mut self,
        other: F,
    ) -> Result<(), Error> {
        let zz = ZZ::from_fieldelement(other)?;
        self.ctxt_add_by_constant_inplace(&zz)
    }

    pub fn ctxt_sub_by_field_element_inplace<F: PrimeField>(
        &mut self,
        other: F,
    ) -> Result<(), Error> {
        let zz = ZZ::from_fieldelement(other)?;
        self.ctxt_sub_by_constant_inplace(&zz)
    }

    pub fn ctxt_sub_from_field_element_inplace<F: PrimeField>(
        &mut self,
        other: F,
    ) -> Result<(), Error> {
        let zz = ZZ::from_fieldelement(other)?;
        self.ctxt_sub_from_constant_inplace(&zz)
    }

    pub fn ctxt_mul_by_field_element_inplace<F: PrimeField>(
        &mut self,
        other: F,
    ) -> Result<(), Error> {
        let zz = ZZ::from_fieldelement(other)?;
        self.ctxt_mul_by_constant_inplace(&zz)
    }
}

impl Drop for Ctxt {
    fn drop(&mut self) {
        self.destroy().expect("Ctxt destroy failed");
    }
}

// Arithmetic

impl Add for &Ctxt {
    type Output = Ctxt;

    fn add(self, other: &Ctxt) -> Ctxt {
        self.ctxt_add(other).expect("Add failed")
    }
}

impl Sub for &Ctxt {
    type Output = Ctxt;

    fn sub(self, other: &Ctxt) -> Ctxt {
        self.ctxt_sub(other).expect("Sub failed")
    }
}

impl Neg for &Ctxt {
    type Output = Ctxt;

    fn neg(self) -> Ctxt {
        self.ctxt_negate().expect("Negate failed")
    }
}

impl Mul for &Ctxt {
    type Output = Ctxt;

    fn mul(self, other: &Ctxt) -> Ctxt {
        self.ctxt_mul(other).expect("Mul failed")
    }
}

// Arithmetic in place

impl AddAssign<&Ctxt> for Ctxt {
    fn add_assign(&mut self, other: &Ctxt) {
        self.ctxt_add_inplace(other).expect("AddAssign failed")
    }
}

impl SubAssign<&Ctxt> for Ctxt {
    fn sub_assign(&mut self, other: &Ctxt) {
        self.ctxt_sub_inplace(other).expect("SubAssign failed")
    }
}

impl MulAssign<&Ctxt> for Ctxt {
    fn mul_assign(&mut self, other: &Ctxt) {
        self.ctxt_mul_inplace(other).expect("MulAssign failed")
    }
}

// Arithmetic with constants

impl Add<&ZZ> for &Ctxt {
    type Output = Ctxt;

    fn add(self, other: &ZZ) -> Ctxt {
        self.ctxt_add_by_constant(other)
            .expect("Add constant failed")
    }
}

impl Sub<&ZZ> for &Ctxt {
    type Output = Ctxt;

    fn sub(self, other: &ZZ) -> Ctxt {
        self.ctxt_sub_by_constant(other)
            .expect("Sub constant failed")
    }
}

impl Sub<&Ctxt> for &ZZ {
    type Output = Ctxt;

    fn sub(self, other: &Ctxt) -> Ctxt {
        other
            .ctxt_sub_from_constant(self)
            .expect("Sub from constant failed")
    }
}

impl Mul<&ZZ> for &Ctxt {
    type Output = Ctxt;

    fn mul(self, other: &ZZ) -> Ctxt {
        self.ctxt_mul_by_constant(other)
            .expect("Mul constant failed")
    }
}

// Arithmetic with packed constants

impl Add<&EncodedPtxt> for &Ctxt {
    type Output = Ctxt;

    fn add(self, other: &EncodedPtxt) -> Ctxt {
        self.ctxt_add_by_packed_constant(other)
            .expect("Add packed constant failed")
    }
}

impl Sub<&EncodedPtxt> for &Ctxt {
    type Output = Ctxt;

    fn sub(self, other: &EncodedPtxt) -> Ctxt {
        self.ctxt_sub_by_packed_constant(other)
            .expect("Sub packed constant failed")
    }
}

impl Sub<&Ctxt> for &EncodedPtxt {
    type Output = Ctxt;

    fn sub(self, other: &Ctxt) -> Ctxt {
        other
            .ctxt_sub_from_packed_constant(self)
            .expect("Sub from packed constant failed")
    }
}

impl Mul<&EncodedPtxt> for &Ctxt {
    type Output = Ctxt;

    fn mul(self, other: &EncodedPtxt) -> Ctxt {
        self.ctxt_mul_by_packed_constant(other)
            .expect("Mul packed constant failed")
    }
}

// Arithmetic with constants in place

impl AddAssign<&ZZ> for Ctxt {
    fn add_assign(&mut self, other: &ZZ) {
        self.ctxt_add_by_constant_inplace(other)
            .expect("AddAssign constant failed")
    }
}

impl SubAssign<&ZZ> for Ctxt {
    fn sub_assign(&mut self, other: &ZZ) {
        self.ctxt_sub_by_constant_inplace(other)
            .expect("SubAssign constant failed")
    }
}

impl MulAssign<&ZZ> for Ctxt {
    fn mul_assign(&mut self, other: &ZZ) {
        self.ctxt_mul_by_constant_inplace(other)
            .expect("MulAssign constant failed")
    }
}

// Arithmetic with constants

impl<F: PrimeField> Add<F> for &Ctxt {
    type Output = Ctxt;

    fn add(self, other: F) -> Ctxt {
        self.ctxt_add_by_field_element(other)
            .expect("Add field_element failed")
    }
}

impl<F: PrimeField> Sub<F> for &Ctxt {
    type Output = Ctxt;

    fn sub(self, other: F) -> Ctxt {
        self.ctxt_sub_by_field_element(other)
            .expect("Sub field_element failed")
    }
}

impl<F: PrimeField> Mul<F> for &Ctxt {
    type Output = Ctxt;

    fn mul(self, other: F) -> Ctxt {
        self.ctxt_mul_by_field_element(other)
            .expect("Mul field_element failed")
    }
}

// Arithmetic with field_elements in place

impl<F: PrimeField> AddAssign<F> for Ctxt {
    fn add_assign(&mut self, other: F) {
        self.ctxt_add_by_field_element_inplace(other)
            .expect("AddAssign field_element failed")
    }
}

impl<F: PrimeField> SubAssign<F> for Ctxt {
    fn sub_assign(&mut self, other: F) {
        self.ctxt_sub_by_field_element_inplace(other)
            .expect("SubAssign field_element failed")
    }
}

impl<F: PrimeField> MulAssign<F> for Ctxt {
    fn mul_assign(&mut self, other: F) {
        self.ctxt_mul_by_field_element_inplace(other)
            .expect("MulAssign field_element failed")
    }
}

impl Clone for Ctxt {
    fn clone(&self) -> Self {
        self.ctxt_clone().expect("Ctxt clone failed")
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

    const TESTRUNS: usize = 10;

    #[test]
    fn build_ctxt() {
        let p = ZZ::char::<ark_bn254::Fr>().unwrap();
        let context = Context::build(32109, &p, 700).unwrap();
        let seckey = SecKey::build(&context).unwrap();
        let pubkey = PubKey::from_seckey(&seckey).unwrap();
        let mut rng = thread_rng();
        let zz = ZZ::from_fieldelement(ark_bn254::Fr::rand(&mut rng)).unwrap();
        let mut ctxt = pubkey.encrypt(&zz).unwrap();
        ctxt.destroy().unwrap(); // Is also called in drop
    }

    #[test]
    fn ctxt_get_noise_budget() {
        let p = ZZ::char::<ark_bn254::Fr>().unwrap();
        let context = Context::build(32109, &p, 700).unwrap();
        let seckey = SecKey::build(&context).unwrap();
        let pubkey = PubKey::from_seckey(&seckey).unwrap();
        let mut rng = thread_rng();
        let zz = ZZ::from_fieldelement(ark_bn254::Fr::rand(&mut rng)).unwrap();
        let ctxt = pubkey.encrypt(&zz).unwrap();
        let _noise = ctxt.noise_budget().unwrap();
    }

    #[test]
    fn ctxt_clone() {
        let p = ZZ::char::<ark_bn254::Fr>().unwrap();
        let context = Context::build(32109, &p, 700).unwrap();
        let seckey = SecKey::build(&context).unwrap();
        let pubkey = PubKey::from_seckey(&seckey).unwrap();
        let mut rng = thread_rng();
        for _ in 0..TESTRUNS {
            let input = ark_bn254::Fr::rand(&mut rng);
            let zz = ZZ::from_fieldelement(input).unwrap();
            let mut ctxt = pubkey.encrypt(&zz).unwrap();
            let ctxt_clone = ctxt.clone();
            ctxt += &ctxt_clone;
            ctxt.destroy().unwrap();
            let clone_ = seckey
                .decrypt_fieldelement::<ark_bn254::Fr>(&ctxt_clone)
                .unwrap();
            assert_eq!(clone_, input);
        }
    }

    #[test]
    fn ctxt_arithmetic_test() {
        let p = ZZ::char::<ark_bn254::Fr>().unwrap();
        let context = Context::build(32109, &p, 700).unwrap();
        let seckey = SecKey::build(&context).unwrap();
        let pubkey = PubKey::from_seckey(&seckey).unwrap();
        let mut rng = thread_rng();
        for _ in 0..TESTRUNS {
            let a = ark_bn254::Fr::rand(&mut rng);
            let b = ark_bn254::Fr::rand(&mut rng);
            let ctxt_a = pubkey.encrypt_fieldelement(a).unwrap();
            let ctxt_b = pubkey.encrypt_fieldelement(b).unwrap();

            let ctxt_add = &ctxt_a + &ctxt_b;
            let ctxt_sub = &ctxt_a - &ctxt_b;
            let ctxt_neg = -&ctxt_a;
            let ctxt_mul = &ctxt_a * &ctxt_b;

            let add = seckey
                .decrypt_fieldelement::<ark_bn254::Fr>(&ctxt_add)
                .unwrap();
            let sub = seckey
                .decrypt_fieldelement::<ark_bn254::Fr>(&ctxt_sub)
                .unwrap();
            let neg = seckey
                .decrypt_fieldelement::<ark_bn254::Fr>(&ctxt_neg)
                .unwrap();
            let mul = seckey
                .decrypt_fieldelement::<ark_bn254::Fr>(&ctxt_mul)
                .unwrap();

            assert_eq!(add, a + b);
            assert_eq!(sub, a - b);
            assert_eq!(neg, -a);
            assert_eq!(mul, a * b);
        }
    }

    #[test]
    fn ctxt_arithmetic_inplace_test() {
        let p = ZZ::char::<ark_bn254::Fr>().unwrap();
        let context = Context::build(32109, &p, 700).unwrap();
        let seckey = SecKey::build(&context).unwrap();
        let pubkey = PubKey::from_seckey(&seckey).unwrap();
        let mut rng = thread_rng();
        for _ in 0..TESTRUNS {
            let a = ark_bn254::Fr::rand(&mut rng);
            let b = ark_bn254::Fr::rand(&mut rng);
            let ctxt_a = pubkey.encrypt_fieldelement(a).unwrap();
            let ctxt_b = pubkey.encrypt_fieldelement(b).unwrap();

            let mut ctxt_add = ctxt_a.clone();
            ctxt_add += &ctxt_b;
            let mut ctxt_sub = ctxt_a.clone();
            ctxt_sub -= &ctxt_b;
            let mut ctxt_neg = ctxt_a.clone();
            ctxt_neg.negate_inplace().unwrap();
            let mut ctxt_mul = ctxt_a.clone();
            ctxt_mul *= &ctxt_b;

            let add = seckey
                .decrypt_fieldelement::<ark_bn254::Fr>(&ctxt_add)
                .unwrap();
            let sub = seckey
                .decrypt_fieldelement::<ark_bn254::Fr>(&ctxt_sub)
                .unwrap();
            let neg = seckey
                .decrypt_fieldelement::<ark_bn254::Fr>(&ctxt_neg)
                .unwrap();
            let mul = seckey
                .decrypt_fieldelement::<ark_bn254::Fr>(&ctxt_mul)
                .unwrap();

            assert_eq!(add, a + b);
            assert_eq!(sub, a - b);
            assert_eq!(neg, -a);
            assert_eq!(mul, a * b);
        }
    }

    #[test]
    fn ctxt_arithmetic_with_const_field_test() {
        let p = ZZ::char::<ark_bn254::Fr>().unwrap();
        let context = Context::build(32109, &p, 700).unwrap();
        let seckey = SecKey::build(&context).unwrap();
        let pubkey = PubKey::from_seckey(&seckey).unwrap();
        let mut rng = thread_rng();
        for _ in 0..TESTRUNS {
            let a = ark_bn254::Fr::rand(&mut rng);
            let b = ark_bn254::Fr::rand(&mut rng);
            let ctxt_a = pubkey.encrypt_fieldelement(a).unwrap();

            let ctxt_add = &ctxt_a + b;
            let ctxt_sub = &ctxt_a - b;
            let ctxt_sub2 = ctxt_a.ctxt_sub_from_field_element(b).unwrap();
            let ctxt_mul = &ctxt_a * b;

            let add = seckey
                .decrypt_fieldelement::<ark_bn254::Fr>(&ctxt_add)
                .unwrap();
            let sub = seckey
                .decrypt_fieldelement::<ark_bn254::Fr>(&ctxt_sub)
                .unwrap();
            let sub2 = seckey
                .decrypt_fieldelement::<ark_bn254::Fr>(&ctxt_sub2)
                .unwrap();
            let mul = seckey
                .decrypt_fieldelement::<ark_bn254::Fr>(&ctxt_mul)
                .unwrap();

            assert_eq!(add, a + b);
            assert_eq!(sub, a - b);
            assert_eq!(sub2, b - a);
            assert_eq!(mul, a * b);
        }
    }

    #[test]
    fn ctxt_arithmetic_with_const_field_inplace_test() {
        let p = ZZ::char::<ark_bn254::Fr>().unwrap();
        let context = Context::build(32109, &p, 700).unwrap();
        let seckey = SecKey::build(&context).unwrap();
        let pubkey = PubKey::from_seckey(&seckey).unwrap();
        let mut rng = thread_rng();
        for _ in 0..TESTRUNS {
            let a = ark_bn254::Fr::rand(&mut rng);
            let b = ark_bn254::Fr::rand(&mut rng);
            let ctxt_a = pubkey.encrypt_fieldelement(a).unwrap();

            let mut ctxt_add = ctxt_a.clone();
            ctxt_add += b;
            let mut ctxt_sub = ctxt_a.clone();
            ctxt_sub -= b;
            let mut ctxt_sub2 = ctxt_a.clone();
            ctxt_sub2.ctxt_sub_from_field_element_inplace(b).unwrap();
            let mut ctxt_mul = ctxt_a.clone();
            ctxt_mul *= b;

            let add = seckey
                .decrypt_fieldelement::<ark_bn254::Fr>(&ctxt_add)
                .unwrap();
            let sub = seckey
                .decrypt_fieldelement::<ark_bn254::Fr>(&ctxt_sub)
                .unwrap();
            let sub2 = seckey
                .decrypt_fieldelement::<ark_bn254::Fr>(&ctxt_sub2)
                .unwrap();
            let mul = seckey
                .decrypt_fieldelement::<ark_bn254::Fr>(&ctxt_mul)
                .unwrap();

            assert_eq!(add, a + b);
            assert_eq!(sub, a - b);
            assert_eq!(sub2, b - a);
            assert_eq!(mul, a * b);
        }
    }
}
