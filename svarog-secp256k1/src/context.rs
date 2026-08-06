use rand::{rand_core::UnwrapErr, rngs::OsRng};
use secp256k1_sys::{self as ffi, CPtr};
use std::{cell::RefCell, ptr::NonNull};

const FLAGS: ffi::types::c_uint = ffi::SECP256K1_START_SIGN | ffi::SECP256K1_START_VERIFY;

/// The secp256k1 engine, used to execute all signature operations.
pub struct Context(pub(crate) NonNull<ffi::Context>);

impl Context {
    /// create a full-featured context.
    pub fn new() -> Self {
        #[cfg(target_arch = "wasm32")]
        ffi::types::sanity_checks_for_wasm();

        let ctx = unsafe { ffi::secp256k1_context_create(FLAGS) };
        let mut ctx = Context(ctx);

        // `OsRng` rather than `rand::rng()`: the latter is a per-thread
        // ChaCha12 cache that is not reseeded on `fork(2)`.
        #[cfg(not(target_arch = "wasm32"))]
        ctx.randomize(&mut UnwrapErr(OsRng));

        ctx
    }

    /// (Re)randomizes the Secp256k1 context for extra sidechannel resistance given 32 bytes of
    /// cryptographically-secure random data;
    /// see comment in libsecp256k1 commit d2275795f by Gregory Maxwell.
    pub fn randomize<R: rand::Rng + ?Sized>(&mut self, rng: &mut R) {
        let mut seed = [0u8; 32];
        rng.fill_bytes(&mut seed);
        unsafe {
            let err = ffi::secp256k1_context_randomize(self.0, seed.as_c_ptr());
            // This function cannot fail; it has an error return for future-proofing.
            // We do not expose this error since it is impossible to hit, and we have
            // precedent for not exposing impossible errors (for example in
            // `PublicKey::from_secret_key` where it is impossible to create an invalid
            // secret key through the API.)
            // However, if this DOES fail, the result is potentially weaker side-channel
            // resistance, which is deadly and undetectable, so we take out the entire
            // thread to be on the safe side.
            assert_eq!(err, 1);
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { ffi::secp256k1_context_destroy(self.0) }
    }
}

thread_local! {
    static CTX: RefCell<Option<Context>> = RefCell::new(None);
}

pub(crate) fn thlocal_ctx() -> *const ffi::Context {
    CTX.with(|obj| {
        let mut instance = obj.borrow_mut();
        if instance.is_none() {
            *instance = Some(Context::new());
        }
        instance.as_ref().unwrap().0.as_ptr()
    })
}
