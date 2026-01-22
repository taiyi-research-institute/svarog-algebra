use std::sync::LazyLock;

use curve_abstract::TrCurve;
use curve25519_dalek::{Scalar as EdwardsScalar, edwards::SubgroupPoint};
use group::Group;
use rug::{Integer, integer::Order};
use serde::{Deserialize, Serialize};

use crate::{Point, Scalar};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Curve25519 {}

impl TrCurve for Curve25519 {
    type PointT = Point;
    type ScalarT = Scalar;

    fn curve_order_bytes() -> &'static [u8] {
        #[rustfmt::skip]
        const CURVE_ORDER: [u8; 32] = [
            0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x14, 0xde, 0xf9, 0xde, 0xa2, 0xf7, 0x9c, 0xd6,
            0x58, 0x12, 0x63, 0x1a, 0x5c, 0xf5, 0xd3, 0xed,
        ];
        &CURVE_ORDER
    }

    fn field_order_bytes() -> &'static [u8] {
        #[rustfmt::skip]
        const FIELD_ORDER: [u8; 32] = [
            0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xed,
        ];
        &FIELD_ORDER
    }

    fn curve_order() -> &'static Integer {
        static N: LazyLock<Integer> =
            LazyLock::new(|| Integer::from_digits(Curve25519::curve_order_bytes(), Order::Msf));
        return &N;
    }

    fn field_order() -> &'static Integer {
        static P: LazyLock<Integer> =
            LazyLock::new(|| Integer::from_digits(Curve25519::field_order_bytes(), Order::Msf));
        return &P;
    }

    fn generator() -> &'static Self::PointT {
        static G: LazyLock<Point> = LazyLock::new(|| Point(SubgroupPoint::generator()));
        &G
    }

    fn identity() -> &'static Self::PointT {
        static ID: LazyLock<Point> = LazyLock::new(|| Point(SubgroupPoint::identity()));
        &ID
    }

    fn zero() -> &'static Self::ScalarT {
        static ZERO: Scalar = Scalar(EdwardsScalar::ZERO);
        &ZERO
    }

    fn one() -> &'static Self::ScalarT {
        static ONE: Scalar = Scalar(EdwardsScalar::ONE);
        &ONE
    }
}
