//! Exercise both libraries in one executable to detect native symbol collisions.

macro_rules! public_key {
    ($ffi:ident) => {{
        let mut secret = [0u8; 32];
        secret[31] = 1;
        let mut bytes = [0u8; 33];
        let mut len = bytes.len();
        // Each library owns its context and public key. Never exchange FFI values
        // between the two independently compiled libraries.
        unsafe {
            let ctx = $ffi::secp256k1_context_create($ffi::SECP256K1_START_SIGN);
            let mut key = $ffi::PublicKey::new();
            assert_eq!(
                $ffi::secp256k1_ec_pubkey_create(ctx.as_ptr(), &mut key, secret.as_ptr()),
                1
            );
            assert_eq!(
                $ffi::secp256k1_ec_pubkey_serialize(
                    ctx.as_ptr(),
                    bytes.as_mut_ptr(),
                    &mut len,
                    &key,
                    $ffi::SECP256K1_SER_COMPRESSED,
                ),
                1
            );
            $ffi::secp256k1_context_destroy(ctx);
        }
        assert_eq!(len, 33);
        bytes
    }};
}

#[test]
fn upstream_and_fork_can_link_and_execute_together() {
    use svarog_secp256k1_sys as fork;
    use upstream_secp256k1_sys as upstream;

    let expected = [
        0x02, 0x79, 0xbe, 0x66, 0x7e, 0xf9, 0xdc, 0xbb, 0xac, 0x55, 0xa0, 0x62, 0x95, 0xce, 0x87,
        0x0b, 0x07, 0x02, 0x9b, 0xfc, 0xdb, 0x2d, 0xce, 0x28, 0xd9, 0x59, 0xf2, 0x81, 0x5b, 0x16,
        0xf8, 0x17, 0x98,
    ];
    assert_eq!(public_key!(upstream), expected);
    assert_eq!(public_key!(fork), expected);
    let infinity = fork::Gej::new_infinity();
    assert_eq!(unsafe { fork::svarog_gej_is_infinity(&infinity) }, 1);
}
