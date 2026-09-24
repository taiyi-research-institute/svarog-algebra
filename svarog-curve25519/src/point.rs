use std::iter::Sum;

use curve_abstract::{TrCurve, TrPoint, TrScalar};
use curve25519_dalek::{EdwardsPoint, constants::ED25519_BASEPOINT_TABLE, edwards::SubgroupPoint};
use group::GroupEncoding;
use serde::{Deserialize, Serialize};

use crate::{Curve25519, Scalar};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Point(pub SubgroupPoint);

impl Default for Point {
    fn default() -> Self {
        Curve25519::identity().clone()
    }
}

impl Serialize for Point {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let ep = EdwardsPoint::from(self.0);
        ep.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Point {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use group::cofactor::CofactorGroup;
        use serde::de::Error;

        let ep = EdwardsPoint::deserialize(deserializer)?;
        let obj = ep.into_subgroup();
        if obj.is_some().into() {
            Ok(Self(obj.unwrap()))
        } else {
            Err(Error::custom(
                "The EdwardPoint is not in the prime-order subgroup",
            ))
        }
    }
}

impl TrPoint<Curve25519> for Point {
    #[inline]
    fn new_from_bytes(buf: &[u8]) -> Result<Self, &str> {
        if buf.len() != 32 {
            return Err("Attempted to parse a buffer (of length != 32) into Point");
        }
        let buf: [u8; 32] = buf.try_into().unwrap();
        let sgp = SubgroupPoint::from_bytes(&buf);
        if sgp.is_some().into() {
            Ok(Self(sgp.unwrap()))
        } else {
            Err("The EdwardPoint is not in the prime-order subgroup")
        }
    }

    #[inline]
    fn to_bytes(&self) -> Vec<u8> {
        use group::GroupEncoding;
        let buf = GroupEncoding::to_bytes(&self.0);
        buf.to_vec()
    }

    #[inline]
    fn to_bytes_long(&self) -> Vec<u8> {
        unimplemented!()
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
    fn sum(points: &[&Self]) -> Self {
        let it = points.iter().map(|x| x.0);
        Self(SubgroupPoint::sum(it))
    }

    #[inline]
    fn neg(&self) -> Self {
        Self(-self.0)
    }

    #[inline]
    fn add_gx(&self, x: &Scalar) -> Self {
        let other = Self::new_gx(x);
        self.add(&other)
    }

    #[inline]
    fn sub_gx(&self, x: &Scalar) -> Self {
        let other = Self::new_gx(&x.neg());
        self.add(&other)
    }

    #[inline]
    fn new_gx(x: &Scalar) -> Self {
        use group::cofactor::CofactorGroup;

        let ep = ED25519_BASEPOINT_TABLE * &x.0;
        Self(ep.into_subgroup().unwrap())
    }

    #[inline]
    fn mul_x(&self, x: &Scalar) -> Self {
        Self(self.0 * x.0)
    }
}
