use std::sync::LazyLock;

use curve_abstract::{self as abs, TrCurve};
use rand::RngCore;
use rug::{Integer, integer::Order};
use secp256k1_sys::{self as ffi, CPtr};
use serde::{Deserialize, Serialize};

use crate::{CURVE_ORDER_WORDS, Secp256k1};

#[derive(Clone, Debug, Default)]
pub struct Scalar(pub(crate) [u8; 32]);

impl abs::TrScalar<Secp256k1> for Scalar {
    #[inline]
    fn new(x: i64) -> Scalar {
        let res = if x == 0 {
            Secp256k1::zero().clone()
        } else if x == 1 {
            Secp256k1::one().clone()
        } else if x >= 0 {
            let mut num = [0u8; 32];
            num[24..].copy_from_slice(&x.to_be_bytes());
            Self(num)
        } else {
            let mut num = [0u8; 32];
            num[24..].copy_from_slice(&(-x).to_be_bytes());
            Self(num).neg()
        };
        res
    }

    fn new_rand() -> Self {
        let mut rng = rand::rng();
        let mut num = [0u8; 32];
        rng.fill_bytes(&mut num);
        Self::new_from_bytes(&num)
    }

    // Big-endian, last 32 bytes.
    fn new_from_bytes(buf: &[u8]) -> Self {
        let mut num = [0u8; 32];
        let (dst, src) = if buf.len() >= 32 {
            (0, buf.len() - 32)
        } else {
            (32 - buf.len(), 0)
        };
        if buf.len() > 0 {
            num[dst..].copy_from_slice(&buf[src..]);
        }
        if num.as_ref() > Secp256k1::curve_order_bytes() {
            // ... then we have `num % CURVE_ORDER == num - CURVE_ORDER`.
            let x2 = u64::from_be_bytes(num[16..24].try_into().unwrap());
            let x1 = u64::from_be_bytes(num[24..32].try_into().unwrap());

            let y2 = CURVE_ORDER_WORDS[2];
            let y1 = CURVE_ORDER_WORDS[3];

            let (z1, emprunt) = soustraire_avec_emprunt(x1, y1);
            let x2 = if emprunt { x2 - 1 } else { x2 };
            let (z2, emprunt) = soustraire_avec_emprunt(x2, y2);

            // Note that x3 >= 0xFFFF_FFFF_FFFF_FFFE == y3.
            let z3 = if emprunt { 0u8 } else { 1u8 };

            num = [0u8; 32];
            num[15] = z3;
            num[16..24].copy_from_slice(&z2.to_be_bytes());
            num[24..].copy_from_slice(&z1.to_be_bytes());
        }
        Scalar(num)
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    #[inline]
    fn new_from_int(x: impl Into<rug::Integer>) -> Self {
        static N: LazyLock<Integer> =
            LazyLock::new(|| Integer::from_digits(&Secp256k1::curve_order_bytes(), Order::Msf));
        let x: Integer = x.into();
        let x = x.modulo(&N);
        let buf = x.to_digits::<u8>(rug::integer::Order::Msf);
        return Self::new_from_bytes(&buf);
    }

    #[inline]
    fn to_int(&self) -> rug::Integer {
        let buf = self.to_bytes();
        let x = rug::Integer::from_digits(&buf, rug::integer::Order::Msf);
        return x;
    }

    #[inline]
    fn add(&self, other: &Self) -> Self {
        let mut x = self.clone();
        let success = unsafe {
            ffi::secp256k1_ec_seckey_tweak_add(
                ffi::secp256k1_context_no_precomp,
                x.as_mut_c_ptr(),
                other.as_c_ptr(),
            )
        };
        if success != 1 {
            if self == Secp256k1::zero() {
                return other.clone();
            }
            if other == Secp256k1::zero() {
                return self.clone();
            }
            assert_eq!(success, 1);
        }
        x
    }

    #[inline]
    /// $$-x = \mathtt{ord} - x$$.
    fn neg(&self) -> Self {
        let mut x = self.clone();
        let success = unsafe {
            ffi::secp256k1_ec_seckey_negate(ffi::secp256k1_context_no_precomp, x.as_mut_c_ptr())
        };
        if success != 1 {
            if self == Secp256k1::zero() {
                return Secp256k1::zero().clone();
            }
            assert_eq!(success, 1);
        }
        x
    }

    #[inline]
    fn sub(&self, other: &Self) -> Self {
        self.add(&other.neg())
    }

    #[inline]
    fn mul(&self, other: &Self) -> Self {
        let mut x = self.clone();

        let success = unsafe {
            ffi::secp256k1_ec_seckey_tweak_mul(
                ffi::secp256k1_context_no_precomp,
                x.as_mut_c_ptr(),
                other.as_c_ptr(),
            )
        };
        if success != 1 {
            if self == Secp256k1::zero() || other == Secp256k1::zero() {
                return Secp256k1::zero().clone();
            }
            assert_eq!(success, 1);
        }
        x
    }

    /// Compute $$x^{-1} \pmod n$$ in constant time.
    /// * Less side-channel leak.
    /// * Less efficient average-cases.
    #[inline]
    fn inv_ct(&self) -> Self {
        if self == Secp256k1::zero() {
            return Secp256k1::zero().clone();
        }
        let mut x = self.clone();

        let success = unsafe {
            ffi::secp256k1_ec_seckey_invert_ct(ffi::secp256k1_context_no_precomp, x.as_mut_c_ptr())
        };
        assert_eq!(success, 1);

        x
    }

    /// Compute $$x^{-1} \pmod n$$ in variable time.
    /// * More efficient average-cases.
    /// * More side-channel leak.
    #[inline]
    fn inv_vt(&self) -> Scalar {
        if self == Secp256k1::zero() {
            return Secp256k1::zero().clone();
        }
        let mut x = self.clone();

        let success = unsafe {
            ffi::secp256k1_ec_seckey_invert_vt(ffi::secp256k1_context_no_precomp, x.as_mut_c_ptr())
        };
        assert_eq!(success, 1);

        x
    }
}

impl Serialize for Scalar {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Scalar {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let inner: [u8; 32] = <[u8; 32]>::deserialize(deserializer)?;
        if inner.as_ref() >= Secp256k1::curve_order_bytes() {
            Err(serde::de::Error::custom("buf >= curve_order"))
        } else {
            Ok(Scalar(inner))
        }
    }
}

impl PartialEq for Scalar {
    /// This implementation is designed to be constant time to help prevent side channel attacks.
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let accum = self
            .0
            .iter()
            .zip(&other.0)
            .fold(0, |accum, (a, b)| accum | (a ^ b));
        unsafe { core::ptr::read_volatile(&accum) == 0 }
    }
}

impl Eq for Scalar {}

impl secp256k1_sys::CPtr for Scalar {
    type Target = u8;

    fn as_c_ptr(&self) -> *const Self::Target {
        let Scalar(dat) = self;
        dat.as_ptr()
    }

    fn as_mut_c_ptr(&mut self) -> *mut Self::Target {
        let &mut Scalar(ref mut dat) = self;
        dat.as_mut_ptr()
    }
}

fn soustraire_avec_emprunt(x: u64, y: u64) -> (u64, bool) {
    if y > x {
        let res = u64::MAX - y + x + 1;
        (res, true)
    } else {
        (x - y, false)
    }
}
