use std::fmt;

use curve_abstract::{self as abs, TrCurve, TrScalar};
use secp256k1_sys::{self as ffi, types::c_int};
use serde::{Deserialize, Serialize};

use crate::{Scalar, Secp256k1, thlocal_ctx};

/// An elliptic curve point on secp256k1, stored internally as a jacobian group element.
#[derive(Clone, Copy)]
pub struct Point(pub(crate) ffi::Gej);

impl Default for Point {
    fn default() -> Self {
        Secp256k1::identity().clone()
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        unsafe { ffi::svarog_gej_eq_var(&self.0, &other.0) == 1 }
    }
}

impl Eq for Point {}

impl abs::TrPoint<Secp256k1> for Point {
    #[inline]
    fn new_from_bytes(buf: &[u8]) -> Result<Self, &str> {
        Self::new_from_bytes(buf)
    }

    #[inline]
    fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes33()
    }

    #[inline]
    fn to_bytes_long(&self) -> Vec<u8> {
        self.to_bytes65()
    }

    #[inline]
    fn add(&self, other: &Self) -> Self {
        let mut r = ffi::Gej::new_infinity();
        unsafe { ffi::svarog_gej_add_var(&mut r, &self.0, &other.0, core::ptr::null_mut()) };
        Point(r)
    }

    #[inline]
    fn sum(points: &[&Self]) -> Self {
        let mut r = ffi::Gej::new_infinity();
        for p in points {
            let mut tmp = ffi::Gej::new_infinity();
            unsafe { ffi::svarog_gej_add_var(&mut tmp, &r, &p.0, core::ptr::null_mut()) };
            r = tmp;
        }
        Point(r)
    }

    #[inline]
    fn sub(&self, other: &Self) -> Self {
        self.add(&other.neg())
    }

    #[inline]
    fn neg(&self) -> Self {
        let mut r = ffi::Gej::new_infinity();
        unsafe { ffi::svarog_gej_neg(&mut r, &self.0) };
        Point(r)
    }

    #[inline]
    fn add_gx(&self, x: &Scalar) -> Self {
        let gx = Self::new_gx(x);
        self.add(&gx)
    }

    #[inline]
    fn sub_gx(&self, x: &Scalar) -> Self {
        let gx = Self::new_gx(&x.neg());
        self.add(&gx)
    }

    #[inline]
    fn new_gx(x: &Scalar) -> Self {
        if x == Secp256k1::zero() {
            return Secp256k1::identity().clone();
        }
        let cs = x.to_cscalar();
        let mut r = ffi::Gej::new_infinity();
        unsafe { ffi::svarog_ecmult_gen(thlocal_ctx(), &mut r, &cs) };
        Point(r)
    }

    #[inline]
    fn mul_x(&self, other: &Scalar) -> Point {
        if other == Secp256k1::zero() {
            return Secp256k1::identity().clone();
        }
        let cs = other.to_cscalar();
        let mut r = ffi::Gej::new_infinity();
        unsafe { ffi::svarog_ecmult_const_gej(&mut r, &self.0, &cs) };
        Point(r)
    }
}

impl Serialize for Point {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_bytes33().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Point {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes: Vec<u8> = Vec::<u8>::deserialize(deserializer)?;
        let point =
            Self::new_from_bytes(&bytes).map_err(|e| serde::de::Error::custom(e.to_string()))?;
        Ok(point)
    }
}

impl Point {
    #[rustfmt::skip]
    pub(crate) const ID_BYTES33: [u8; 33] = [
        0x02,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    #[rustfmt::skip]
    pub(crate) const ID_BYTES65: [u8; 65] = [
        0x04,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    fn new_from_bytes(data: &[u8]) -> Result<Point, &'static str> {
        if data.len() != 33 && data.len() != 65 {
            return Err("Invalid length of point bytes");
        }
        // Check for identity encoding
        if data.len() == 33 && data == &Self::ID_BYTES33 {
            return Ok(Secp256k1::identity().clone());
        }
        if data.len() == 65 && data == &Self::ID_BYTES65 {
            return Ok(Secp256k1::identity().clone());
        }
        let mut r = ffi::Gej::new_infinity();
        let ok = unsafe {
            ffi::svarog_gej_parse(&mut r, data.as_ptr(), data.len() as c_int)
        };
        if ok != 1 {
            return Err("Invalid point bytes (not on curve).");
        }
        Ok(Point(r))
    }

    fn to_bytes33(&self) -> Vec<u8> {
        if self.is_infinity() {
            return Self::ID_BYTES33.to_vec();
        }
        let mut buf = vec![0u8; 33];
        let mut outlen: c_int = 33;
        let ok = unsafe {
            ffi::svarog_gej_serialize(buf.as_mut_ptr(), &mut outlen, &self.0, 1)
        };
        assert_eq!(ok, 1);
        buf
    }

    fn to_bytes65(&self) -> Vec<u8> {
        if self.is_infinity() {
            return Self::ID_BYTES65.to_vec();
        }
        let mut buf = vec![0u8; 65];
        let mut outlen: c_int = 65;
        let ok = unsafe {
            ffi::svarog_gej_serialize(buf.as_mut_ptr(), &mut outlen, &self.0, 0)
        };
        assert_eq!(ok, 1);
        buf
    }

    #[inline]
    fn is_infinity(&self) -> bool {
        unsafe { ffi::svarog_gej_is_infinity(&self.0) == 1 }
    }
}

impl fmt::LowerHex for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ser = self.to_bytes65();
        for ch in &ser[1..33] {
            write!(f, "{:02x}", *ch)?;
        }
        write!(f, "_")?;
        for ch in &ser[33..] {
            write!(f, "{:02x}", *ch)?;
        }
        Ok(())
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::LowerHex::fmt(self, f)
    }
}

impl fmt::Debug for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::LowerHex::fmt(self, f)
    }
}
