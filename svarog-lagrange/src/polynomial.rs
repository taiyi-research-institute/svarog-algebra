#![allow(nonstandard_style)]
use std::collections::{HashMap, HashSet};

use curve_abstract::{TrCurve, TrPoint, TrScalar};
use erreur::*;
use rug::Integer;

use super::ShamirScheme;

impl<C: TrCurve + 'static> VerifiableSecretSharing<C> for C {
    fn generate_shares(
        ui: &C::ScalarT,
        omega_k: &HashSet<usize>,
        th: usize,
    ) -> (Vec<C::PointT>, HashMap<usize, C::ScalarT>) {
        assert!(th >= 1 && th <= omega_k.len());

        // generate $$f_i(X)$$, a secret, ephemeral polynomial.
        let mut fiX: Vec<C::ScalarT> = vec![C::zero().clone(); th];
        fiX[0] = ui.clone();
        for k in 1..th {
            fiX[k] = C::ScalarT::new_rand();
        }

        // commit $$f_i(X)$$ as $$F_i(X)$$.
        let mut FiX: Vec<C::PointT> = Vec::with_capacity(th);
        for coef in &fiX {
            FiX.push(C::PointT::new_gx(coef));
        }

        // evaluate $$f_i(j)$$ for every j in omega_k.
        let mut fij_map: HashMap<usize, C::ScalarT> = HashMap::with_capacity(omega_k.len());
        for j in omega_k {
            let j_scalar = C::ScalarT::new(*j as i64);
            let v = Self::eval_poly(&j_scalar, &fiX);
            fij_map.insert(*j, v);
        }

        (FiX, fij_map)
    }

    fn eval_poly(x: &C::ScalarT, poly: &[C::ScalarT]) -> C::ScalarT {
        let mut res = C::zero().clone();
        for k in (0..poly.len()).rev() {
            res = res.mul(x).add(&poly[k]);
        }
        res
    }

    fn eval_polycom(x: &C::ScalarT, polycom: &[C::PointT]) -> C::PointT {
        let mut res = C::identity().clone();
        for k in (0..polycom.len()).rev() {
            res = res.mul_x(x).add(&polycom[k]);
        }
        res
    }

    fn verify_fj_at_i(
        i: usize,
        xji_map: &HashMap<usize, C::ScalarT>,
        vss_scheme: &ShamirScheme<C>,
    ) -> Resultat<()> {
        let i_scalar = C::ScalarT::new(i as i64);

        for (j, fji) in xji_map {
            assert_throw!(vss_scheme.contains_key(j));
            let lhs = C::PointT::new_gx(fji);
            let rhs = Self::eval_polycom(&i_scalar, &vss_scheme[j]);
            assert_throw!(
                lhs == rhs,
                format!("Math broken when verifying f_j(i), i={}, j={}", i, j)
            );
        }
        Ok(())
    }

    fn eval_xi_com(i: usize, vss_scheme: &ShamirScheme<C>) -> C::PointT {
        let mut xig = C::identity().clone();
        let i_scalar = C::ScalarT::new(i as i64);
        for (_, polycom) in vss_scheme {
            let Fji = Self::eval_polycom(&i_scalar, polycom);
            xig = xig.add(&Fji);
        }
        xig
    }

    fn lagrange_lambda(i: usize, signers: &HashSet<usize>) -> C::ScalarT {
        let i_scalar = C::ScalarT::new(i as i64);
        let mut num = C::one().clone();
        let mut den = C::one().clone();

        for j in signers {
            if i == *j {
                continue;
            }
            let j_scalar = C::ScalarT::new(*j as i64);
            num = num.mul(&j_scalar);
            den = den.mul(&j_scalar.sub(&i_scalar));
        }

        den = den.inv_ct(); // surely invertible.
        num.mul(&den)
    }
}

pub trait VerifiableSecretSharing<C: TrCurve> {
    /// Generate VSS shares.
    ///
    /// Math.
    ///
    /// * Generate $$f_i(X) = a_0X^0 + a_1X^1 + \dots + a_{t-1}X^{t-1}$$,
    /// Here, $$a_0 := u_i$$, and other coefficients are randomly generated.
    /// * Evalute $$f_i(j)$$ for any $$j\in \Omega_k$$.
    /// Here, $$\Omega_k$$ is the set of keygen player ids.
    ///
    /// Parameters.
    ///
    /// * `ui` - Player's additive share.
    /// * `omega_k` - Set of keygen player ids.
    /// * `th` - Minimum number of players when signing.
    ///
    /// Returns.
    ///
    /// * `Vec<C::PointT>` - This is essentially $$F_j(X)=A_0X^0 + A_1X^1 + \dots + A_{t-1}X^{t-1}$$.
    /// * `HashMap<usize, C::ScalarT>` - The sequence $$f_i(j)$$ organized as a `HashMap`, whose key is $$j$$.
    ///
    /// Notes.
    ///
    /// We use capital X to refer to an "intermediate".
    ///
    /// An intermediate is a symbol with no predetermined value.
    ///
    /// We can replace an intermidiate with anything
    /// whose "power by integer >= 0" and "multiplication by coefficient" is properly defined.
    ///
    /// Such replacement is called "evaluation".
    fn generate_shares(
        ui: &C::ScalarT,
        omega_k: &HashSet<usize>,
        th: usize,
    ) -> (Vec<C::PointT>, HashMap<usize, C::ScalarT>);

    /// Evaluate a polynomial at x, modulo curve order.
    ///
    /// Math.
    ///
    /// Evaluate $$f(X) = a_0X^0+a_1X^1+ \dots + a_{t-1}X^{t-1}$$,
    /// at $$X=x$$, using Qin Jiushao / Horner's method.
    ///
    /// Parameters.
    ///
    /// * `x` - Input to $$f(X)$$.
    /// * `poly` - Expressed as $$a_0, a_1, \dots, a_{t-1}$$.
    ///
    /// Returns.
    ///
    /// * `C::ScalarT` - $$f(x)$$.
    fn eval_poly(x: &C::ScalarT, poly: &[C::ScalarT]) -> C::ScalarT;

    /// Evaluate a polynomial commitment at x.
    ///
    /// Math.
    ///
    /// Evaluate $$F(X) = A_0X^0+A_1X^1+ \dots + A_{t-1}X^{t-1}$$,
    /// at $$X=x$$, using Qin Jiushao / Horner's method.
    ///
    /// Parameters.
    ///
    /// * `x` - Input to $$f(x)$$.
    /// * `polycom` - Expressed as $$A_0, A_1, \dots, A_{t-1}$$.
    ///
    /// Returns.
    ///
    /// * `C::PointT` - $$F(x)$$.
    fn eval_polycom(x: &C::ScalarT, polycom: &[C::PointT]) -> C::PointT;

    /// Verify that all other players honestly evaluated their polynomials.
    ///
    /// Math.
    ///
    /// Verify that $$f_j(i)G = F_j(i)$$.
    ///
    /// Parameters.
    ///
    /// * `i` - Player ID.
    /// * `xji_map` - $$f_j(i)$$-s from every players to $$i$$, organized as a `HashMap`.
    /// * `vss_scheme` - The set of $$F_j(X)$$ for every keygen player $$j$$.
    ///
    /// Notes.
    ///
    /// `vss_scheme` should be identical among all keygen players, by its math nature.
    fn verify_fj_at_i(
        i: usize,
        xji_map: &HashMap<usize, C::ScalarT>,
        vss_scheme: &ShamirScheme<C>,
    ) -> Resultat<()>;

    /// Evaluate $$x_i G$$ without knowing $$x_i$$.
    ///
    /// Math.
    ///
    /// As $$x_i = \sum_{j\in\Omega_k} f_j(i)$$,
    /// we have $$x_i G = \sum_{j\in\Omega_k} F_j(i)$$.
    ///
    /// Parameters.
    ///
    /// * `i` - Player ID.
    /// * `vss_scheme` - The set of $$F_j(X)$$ for every keygen player $$j$$.
    fn eval_xi_com(i: usize, vss_scheme: &ShamirScheme<C>) -> C::PointT;

    /// Math.
    ///
    /// Evaluate $$\lambda_\mathtt{id}$$, such that
    ///
    /// $$x=\sum\lambda_i x_i$$ for $$i$$ in $$\Omega_s$$.
    ///
    /// Parameters.
    ///
    /// * `i` - Player ID.
    /// * `signers` - The set of signer ids, i.e. $$\Omega_s$$.
    ///
    /// Returns.
    ///
    /// * `C::ScalarT` - $$\lambda_i$$.
    fn lagrange_lambda(i: usize, signers: &HashSet<usize>) -> C::ScalarT;
}
