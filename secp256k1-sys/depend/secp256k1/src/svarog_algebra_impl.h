/**********************************************************************
 * svarog_algebra_impl.h - Non-static wrappers for internal secp256k1
 * group/scalar/field operations. These thin wrappers expose the static
 * internal functions for use via Rust FFI.
 *
 * This file is #include'd at the end of secp256k1.c, where all the
 * static functions are already visible.
 **********************************************************************/

#ifndef SVAROG_ALGEBRA_IMPL_H
#define SVAROG_ALGEBRA_IMPL_H

/* ===== Group element (point) operations ===== */

void svarog_gej_set_infinity(rustsecp256k1_v0_13_gej *r) {
    rustsecp256k1_v0_13_gej_set_infinity(r);
}

int svarog_gej_is_infinity(const rustsecp256k1_v0_13_gej *a) {
    return rustsecp256k1_v0_13_gej_is_infinity(a);
}

int svarog_ge_is_infinity(const rustsecp256k1_v0_13_ge *a) {
    return rustsecp256k1_v0_13_ge_is_infinity(a);
}

/** Convert jacobian → affine (variable time). NOTE: mutates `a`. */
void svarog_ge_set_gej_var(rustsecp256k1_v0_13_ge *r, rustsecp256k1_v0_13_gej *a) {
    rustsecp256k1_v0_13_ge_set_gej_var(r, a);
}

void svarog_gej_set_ge(rustsecp256k1_v0_13_gej *r, const rustsecp256k1_v0_13_ge *a) {
    rustsecp256k1_v0_13_gej_set_ge(r, a);
}

void svarog_ge_set_xy(rustsecp256k1_v0_13_ge *r, const rustsecp256k1_v0_13_fe *x, const rustsecp256k1_v0_13_fe *y) {
    rustsecp256k1_v0_13_ge_set_xy(r, x, y);
}

void svarog_gej_neg(rustsecp256k1_v0_13_gej *r, const rustsecp256k1_v0_13_gej *a) {
    rustsecp256k1_v0_13_gej_neg(r, a);
}

/** Add two jacobian points. rzr may be NULL. */
void svarog_gej_add_var(rustsecp256k1_v0_13_gej *r,
                        const rustsecp256k1_v0_13_gej *a,
                        const rustsecp256k1_v0_13_gej *b,
                        rustsecp256k1_v0_13_fe *rzr) {
    rustsecp256k1_v0_13_gej_add_var(r, a, b, rzr);
}

/** Add jacobian + affine. rzr may be NULL. */
void svarog_gej_add_ge_var(rustsecp256k1_v0_13_gej *r,
                           const rustsecp256k1_v0_13_gej *a,
                           const rustsecp256k1_v0_13_ge *b,
                           rustsecp256k1_v0_13_fe *rzr) {
    rustsecp256k1_v0_13_gej_add_ge_var(r, a, b, rzr);
}

/** Double a jacobian point. rzr may be NULL. */
void svarog_gej_double_var(rustsecp256k1_v0_13_gej *r,
                           const rustsecp256k1_v0_13_gej *a,
                           rustsecp256k1_v0_13_fe *rzr) {
    rustsecp256k1_v0_13_gej_double_var(r, a, rzr);
}

/** Convert affine → compact 64-byte storage. */
void svarog_ge_to_storage(rustsecp256k1_v0_13_ge_storage *r,
                          const rustsecp256k1_v0_13_ge *a) {
    rustsecp256k1_v0_13_ge_to_storage(r, a);
}

/** Convert compact storage → affine (always sets infinity=0). */
void svarog_ge_from_storage(rustsecp256k1_v0_13_ge *r,
                            const rustsecp256k1_v0_13_ge_storage *a) {
    rustsecp256k1_v0_13_ge_from_storage(r, a);
}

/* ===== High-level convenience wrappers (operate on gej directly) ===== */

/** Compare two jacobian points for equality. Handles infinity. */
int svarog_gej_eq_var(const rustsecp256k1_v0_13_gej *a,
                      const rustsecp256k1_v0_13_gej *b) {
    int a_inf = rustsecp256k1_v0_13_gej_is_infinity(a);
    int b_inf = rustsecp256k1_v0_13_gej_is_infinity(b);
    if (a_inf && b_inf) return 1;
    if (a_inf || b_inf) return 0;
    rustsecp256k1_v0_13_ge ga, gb;
    rustsecp256k1_v0_13_gej ca = *a, cb = *b;
    rustsecp256k1_v0_13_ge_set_gej_var(&ga, &ca);
    rustsecp256k1_v0_13_ge_set_gej_var(&gb, &cb);
    return rustsecp256k1_v0_13_ge_eq_var(&ga, &gb);
}

/** Scalar multiply with jacobian input: r = q * a. Handles infinity. */
void svarog_ecmult_const_gej(rustsecp256k1_v0_13_gej *r,
                             const rustsecp256k1_v0_13_gej *a,
                             const rustsecp256k1_v0_13_scalar *q) {
    if (rustsecp256k1_v0_13_gej_is_infinity(a)) {
        rustsecp256k1_v0_13_gej_set_infinity(r);
        return;
    }
    rustsecp256k1_v0_13_ge a_aff;
    rustsecp256k1_v0_13_gej a_copy = *a;
    rustsecp256k1_v0_13_ge_set_gej_var(&a_aff, &a_copy);
    rustsecp256k1_v0_13_ecmult_const(r, &a_aff, q);
}

/** Convert jacobian → 64-byte ge_storage. Returns 0 for infinity. */
int svarog_gej_to_storage(rustsecp256k1_v0_13_ge_storage *r,
                          const rustsecp256k1_v0_13_gej *a) {
    if (rustsecp256k1_v0_13_gej_is_infinity(a)) return 0;
    rustsecp256k1_v0_13_ge a_aff;
    rustsecp256k1_v0_13_gej a_copy = *a;
    rustsecp256k1_v0_13_ge_set_gej_var(&a_aff, &a_copy);
    rustsecp256k1_v0_13_ge_to_storage(r, &a_aff);
    return 1;
}

/** Convert 64-byte ge_storage → jacobian. */
void svarog_gej_from_storage(rustsecp256k1_v0_13_gej *r,
                             const rustsecp256k1_v0_13_ge_storage *a) {
    rustsecp256k1_v0_13_ge tmp;
    rustsecp256k1_v0_13_ge_from_storage(&tmp, a);
    rustsecp256k1_v0_13_gej_set_ge(r, &tmp);
}

/** Serialize jacobian point to compressed (33-byte) or uncompressed (65-byte).
 *  Returns the number of bytes written, or 0 for infinity. */
int svarog_gej_serialize(unsigned char *out, int *outlen,
                         const rustsecp256k1_v0_13_gej *a, int compressed) {
    if (rustsecp256k1_v0_13_gej_is_infinity(a)) return 0;
    rustsecp256k1_v0_13_ge a_aff;
    rustsecp256k1_v0_13_gej a_copy = *a;
    rustsecp256k1_v0_13_ge_set_gej_var(&a_aff, &a_copy);
    rustsecp256k1_v0_13_fe_normalize_var(&a_aff.x);
    rustsecp256k1_v0_13_fe_normalize_var(&a_aff.y);
    unsigned char x[32], y[32];
    rustsecp256k1_v0_13_fe_get_b32(x, &a_aff.x);
    rustsecp256k1_v0_13_fe_get_b32(y, &a_aff.y);
    if (compressed) {
        *outlen = 33;
        out[0] = (y[31] & 1) ? 0x03 : 0x02;
        memcpy(out + 1, x, 32);
    } else {
        *outlen = 65;
        out[0] = 0x04;
        memcpy(out + 1, x, 32);
        memcpy(out + 33, y, 32);
    }
    return 1;
}

/** Deserialize compressed (33-byte) or uncompressed (65-byte) to jacobian.
 *  Returns 1 on success, 0 on failure. */
int svarog_gej_parse(rustsecp256k1_v0_13_gej *r,
                     const unsigned char *input, int inputlen) {
    rustsecp256k1_v0_13_fe x, y;
    if (inputlen == 33) {
        if (input[0] != 0x02 && input[0] != 0x03) return 0;
        if (!rustsecp256k1_v0_13_fe_set_b32_limit(&x, input + 1)) return 0;
        rustsecp256k1_v0_13_ge ge;
        if (!rustsecp256k1_v0_13_ge_set_xo_var(&ge, &x, input[0] == 0x03)) return 0;
        rustsecp256k1_v0_13_gej_set_ge(r, &ge);
        return 1;
    } else if (inputlen == 65) {
        if (input[0] != 0x04 && input[0] != 0x06 && input[0] != 0x07) return 0;
        if (!rustsecp256k1_v0_13_fe_set_b32_limit(&x, input + 1)) return 0;
        if (!rustsecp256k1_v0_13_fe_set_b32_limit(&y, input + 33)) return 0;
        rustsecp256k1_v0_13_ge ge;
        rustsecp256k1_v0_13_ge_set_xy(&ge, &x, &y);
        if (!rustsecp256k1_v0_13_ge_is_valid_var(&ge)) return 0;
        if ((input[0] == 0x06 || input[0] == 0x07) &&
            ((input[0] & 1) != rustsecp256k1_v0_13_fe_is_odd(&y))) return 0;
        rustsecp256k1_v0_13_gej_set_ge(r, &ge);
        return 1;
    }
    return 0;
}

/* ===== Scalar multiplication (low-level) ===== */

/** Constant-time: r = q * a (affine point). */
void svarog_ecmult_const(rustsecp256k1_v0_13_gej *r,
                         const rustsecp256k1_v0_13_ge *a,
                         const rustsecp256k1_v0_13_scalar *q) {
    rustsecp256k1_v0_13_ecmult_const(r, a, q);
}

/** r = a * G (generator). Requires a built context. */
void svarog_ecmult_gen(const rustsecp256k1_v0_13_context *ctx,
                       rustsecp256k1_v0_13_gej *r,
                       const rustsecp256k1_v0_13_scalar *a) {
    rustsecp256k1_v0_13_ecmult_gen(&ctx->ecmult_gen_ctx, r, a);
}

/* ===== Field element operations ===== */

int svarog_fe_set_b32_limit(rustsecp256k1_v0_13_fe *r,
                            const unsigned char *a) {
    return rustsecp256k1_v0_13_fe_set_b32_limit(r, a);
}

void svarog_fe_get_b32(unsigned char *r,
                       const rustsecp256k1_v0_13_fe *a) {
    rustsecp256k1_v0_13_fe_get_b32(r, a);
}

void svarog_fe_normalize_var(rustsecp256k1_v0_13_fe *r) {
    rustsecp256k1_v0_13_fe_normalize_var(r);
}

/* ===== Scalar operations ===== */

void svarog_scalar_set_b32(rustsecp256k1_v0_13_scalar *r,
                           const unsigned char *bin,
                           int *overflow) {
    rustsecp256k1_v0_13_scalar_set_b32(r, bin, overflow);
}

void svarog_scalar_get_b32(unsigned char *bin,
                           const rustsecp256k1_v0_13_scalar *a) {
    rustsecp256k1_v0_13_scalar_get_b32(bin, a);
}

void svarog_scalar_negate(rustsecp256k1_v0_13_scalar *r,
                          const rustsecp256k1_v0_13_scalar *a) {
    rustsecp256k1_v0_13_scalar_negate(r, a);
}

int svarog_scalar_add(rustsecp256k1_v0_13_scalar *r,
                      const rustsecp256k1_v0_13_scalar *a,
                      const rustsecp256k1_v0_13_scalar *b) {
    return rustsecp256k1_v0_13_scalar_add(r, a, b);
}

void svarog_scalar_mul(rustsecp256k1_v0_13_scalar *r,
                       const rustsecp256k1_v0_13_scalar *a,
                       const rustsecp256k1_v0_13_scalar *b) {
    rustsecp256k1_v0_13_scalar_mul(r, a, b);
}

void svarog_scalar_inverse_var(rustsecp256k1_v0_13_scalar *r,
                               const rustsecp256k1_v0_13_scalar *a) {
    rustsecp256k1_v0_13_scalar_inverse_var(r, a);
}

int svarog_scalar_is_zero(const rustsecp256k1_v0_13_scalar *a) {
    return rustsecp256k1_v0_13_scalar_is_zero(a);
}

int svarog_scalar_eq(const rustsecp256k1_v0_13_scalar *a,
                     const rustsecp256k1_v0_13_scalar *b) {
    return rustsecp256k1_v0_13_scalar_eq(a, b);
}

/** High-level: invert a 32-byte big-endian secret key in-place (variable-time). */
int svarog_seckey_inverse(unsigned char *seckey) {
    rustsecp256k1_v0_13_scalar s, inv;
    int overflow;
    rustsecp256k1_v0_13_scalar_set_b32(&s, seckey, &overflow);
    if (rustsecp256k1_v0_13_scalar_is_zero(&s)) return 0;
    rustsecp256k1_v0_13_scalar_inverse_var(&inv, &s);
    rustsecp256k1_v0_13_scalar_get_b32(seckey, &inv);
    return 1;
}

#endif /* SVAROG_ALGEBRA_IMPL_H */
