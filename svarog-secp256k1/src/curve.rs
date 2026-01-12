use std::sync::LazyLock;

use crate::{Point, Scalar};

use curve_abstract::{self as abs, TrPoint};
use rug::{Integer, integer::Order};
use secp256k1_sys as ffi;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Secp256k1 {}

impl abs::TrCurve for Secp256k1 {
    type PointT = Point;
    type ScalarT = Scalar;

    fn curve_order_bytes() -> &'static [u8] {
        #[rustfmt::skip]
        const CURVE_ORDER: [u8; 32] = [
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
            0xba, 0xae, 0xdc, 0xe6, 0xaf, 0x48, 0xa0, 0x3b,
            0xbf, 0xd2, 0x5e, 0x8c, 0xd0, 0x36, 0x41, 0x41
        ];
        &CURVE_ORDER
    }

    fn field_order_bytes() -> &'static [u8] {
        #[rustfmt::skip]
        const FIELD_ORDER: [u8; 32] = [
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xfe, 0xff, 0xff, 0xfc, 0x2f
        ];
        &FIELD_ORDER
    }

    fn curve_order() -> &'static Integer {
        static N: LazyLock<Integer> =
            LazyLock::new(|| Integer::from_digits(Secp256k1::curve_order_bytes(), Order::Msf));
        return &N;
    }

    fn field_order() -> &'static Integer {
        static P: LazyLock<Integer> =
            LazyLock::new(|| Integer::from_digits(Secp256k1::field_order_bytes(), Order::Msf));
        return &P;
    }

    fn generator() -> &'static Point {
        #[rustfmt::skip]
        const GENERATOR_BYTES65: [u8; 65] = [
            0x04,
            // X
            0x79, 0xbe, 0x66, 0x7e, 0xf9, 0xdc, 0xbb, 0xac,
            0x55, 0xa0, 0x62, 0x95, 0xce, 0x87, 0x0b, 0x07,
            0x02, 0x9b, 0xfc, 0xdb, 0x2d, 0xce, 0x28, 0xd9,
            0x59, 0xf2, 0x81, 0x5b, 0x16, 0xf8, 0x17, 0x98,
            // Y
            0x48, 0x3a, 0xda, 0x77, 0x26, 0xa3, 0xc4, 0x65,
            0x5d, 0xa4, 0xfb, 0xfc, 0x0e, 0x11, 0x08, 0xa8,
            0xfd, 0x17, 0xb4, 0x48, 0xa6, 0x85, 0x54, 0x19,
            0x9c, 0x47, 0xd0, 0x8f, 0xfb, 0x10, 0xd4, 0xb8
        ];
        static GENERATOR: LazyLock<Point> =
            LazyLock::new(|| Point::new_from_bytes(&GENERATOR_BYTES65).unwrap());
        &GENERATOR
    }

    fn identity() -> &'static Point {
        static IDENTITY: LazyLock<Point> = LazyLock::new(|| {
            let inner = unsafe { ffi::PublicKey::new() };
            Point(inner)
        });
        &IDENTITY
    }

    fn zero() -> &'static Scalar {
        static ZERO: Scalar = Scalar([0u8; 32]);
        &ZERO
    }

    fn one() -> &'static Scalar {
        #[rustfmt::skip]
        static ONE: Scalar = Scalar([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
        ]);
        &ONE
    }
}

#[rustfmt::skip]
pub(crate) const CURVE_ORDER_WORDS: [u64; 4] = [
    0xFFFF_FFFF_FFFF_FFFF, 0xFFFF_FFFF_FFFF_FFFE,
    0xBAAE_DCE6_AF48_A03B, 0xBFD2_5E8C_D036_4141
];
