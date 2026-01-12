use std::sync::LazyLock;

use curve_abstract::{TrCurve, TrScalar};
use curve25519_dalek::Scalar as EdwardScalar;
use rand::RngCore;
use rug::{Integer, integer::Order};
use serde::{Deserialize, Serialize};

use crate::Curve25519;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Scalar(pub EdwardScalar);

impl TrScalar<Curve25519> for Scalar {
    #[inline]
    fn new(n: i64) -> Self {
        let es: EdwardScalar = if n >= 0 {
            EdwardScalar::from(n as u64)
        } else {
            -EdwardScalar::from(-n as u64)
        };
        Self(es)
    }

    #[inline]
    fn new_rand() -> Self {
        let mut rng = rand::rng();
        let mut buf = [0u8; 64];
        rng.fill_bytes(&mut buf);
        Self::new_from_bytes(&buf)
    }

    // Little-endian, truncate or zero-padding at tail, to 64 bytes.
    #[inline]
    fn new_from_bytes(buf: &[u8]) -> Self {
        let mut buf64 = [0u8; 64];
        let end = if buf.len() >= 64 { 64 } else { buf.len() };
        buf64[..end].copy_from_slice(&buf[..end]);
        let es = EdwardScalar::from_bytes_mod_order_wide(&buf64);
        Scalar(es)
    }

    #[inline]
    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes().to_vec()
    }

    #[inline]
    fn new_from_int(x: impl Into<Integer>) -> Self {
        static N: LazyLock<Integer> =
            LazyLock::new(|| Integer::from_digits(&Curve25519::curve_order_bytes(), Order::Msf));
        let x: Integer = x.into();
        let x = x.modulo(&N);
        let buf = x.to_digits::<u8>(rug::integer::Order::Lsf);
        return Self::new_from_bytes(&buf);
    }

    #[inline]
    fn to_int(&self) -> rug::Integer {
        let buf = self.to_bytes();
        let x = rug::Integer::from_digits(&buf, rug::integer::Order::Lsf);
        return x;
    }

    #[inline]
    fn add(&self, other: &Self) -> Self {
        Self(self.0 + other.0)
    }

    #[inline]
    fn sub(&self, other: &Self) -> Self {
        Self(self.0 - other.0)
    }

    #[inline]
    fn neg(&self) -> Self {
        Self(-self.0)
    }

    #[inline]
    fn mul(&self, other: &Self) -> Self {
        Self(self.0 * other.0)
    }

    #[inline]
    fn inv_ct(&self) -> Self {
        if self.0 == EdwardScalar::ZERO {
            Self(EdwardScalar::ZERO)
        } else {
            Self(self.0.invert())
        }
    }

    #[inline]
    fn inv_vt(&self) -> Self {
        unimplemented!()
    }
}
