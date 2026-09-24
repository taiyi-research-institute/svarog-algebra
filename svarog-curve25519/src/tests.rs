use curve_abstract::{TrPoint, TrScalar};

use crate::{Point, Scalar};

#[test]
#[allow(non_snake_case)]
fn test_sign_verify() {
    use std::io::{Seek, SeekFrom};

    use serde::{Deserialize, Serialize};
    use serde_cbor;
    use tempfile::tempfile;

    let m = Scalar::new(114514);

    // Alice keygen
    let x = Scalar::new_rand();
    let Q = Point::new_gx(&x);
    let r = Scalar::new_rand();
    let R = Point::new_gx(&r);
    let s = r.add(&m.mul(&x));

    // Bob verify
    let sG = Point::new_gx(&s);
    let right = R.add(&Q.mul_x(&m));
    assert_eq!(sG, right);

    // test serde
    #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct Signature(Point, Scalar);

    let sig = Signature(R, s);
    let mut file = tempfile().unwrap();
    serde_cbor::to_writer(&file, &sig).unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();
    let sig2: Signature = serde_cbor::from_reader(&file).unwrap();
    assert_eq!(sig, sig2);
}

#[test]
fn test_fromto_bytes() {
    let x = Scalar::new_rand();
    let b = x.to_bytes();
    assert_eq!(b.len(), 32);
    let y = Scalar::new_from_bytes(&b);
    assert_eq!(x, y);
}
