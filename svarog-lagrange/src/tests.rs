use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

use curve_abstract::{TrCurve, TrPoint, TrScalar};
use rand::Rng;
use rug::{Integer, ops::DivRounding};
use svarog_secp256k1::Scalar;

use crate::*;

#[test]
fn test_vss() {
    use svarog_curve25519::Curve25519;
    use svarog_secp256k1::Secp256k1;

    test_vss_impl::<Secp256k1>();
    test_vss_impl::<Curve25519>();
}

#[allow(nonstandard_style)]
pub fn test_vss_impl<C: TrCurve + 'static>() {
    // generate random threshold, n_keygen, n_sign
    let mut _rng = rand::rng();
    let rng = &mut _rng;
    let mut params: Vec<usize> = vec![
        rng.random_range(3..=20),
        rng.random_range(3..=20),
        rng.random_range(3..=20),
    ];
    params.sort();
    // threshold, n_sign, n_keygen
    let (th, ns, nk) = (params[0], params[1], params[2]);
    let omega_k: HashSet<usize> = (1..=nk).collect();

    // 1. Keygen
    let start = Instant::now();
    let ui_map: HashMap<usize, C::ScalarT> =
        (1..=nk).map(|k| (k, C::ScalarT::new_rand())).collect();
    let mut vss_scheme = ShamirScheme::<C>::new();
    let mut xi_map: HashMap<usize, C::ScalarT> = (1..=nk).map(|k| (k, C::zero().clone())).collect();
    let mut everyi_received_xji_map: HashMap<usize, HashMap<usize, C::ScalarT>> = HashMap::new();
    for i in &omega_k {
        let mut xji_map = HashMap::new();
        for j in &omega_k {
            xji_map.insert(*j, C::zero().clone());
        }
        everyi_received_xji_map.insert(*i, xji_map);
    }
    for i in &omega_k {
        let ui = &ui_map[&i];
        let (FiX, fij_map) = C::generate_shares(ui, &omega_k, th);
        vss_scheme.insert(*i, FiX);
        for (j, fij) in &fij_map {
            let xj = xi_map.get_mut(j).unwrap();
            *xj = xj.add(fij);
            let xij = everyi_received_xji_map
                .get_mut(j)
                .unwrap()
                .get_mut(&i)
                .unwrap();
            *xij = fij.clone();
        }
    }
    let dur = start.elapsed().as_micros();
    println!("keygen elapsed {} us", dur);

    // 2. Verify
    // 2.1. Verify constant terms of FjX for every j.
    let pk0 = {
        let mut pk = C::identity().clone();
        for ui in ui_map.values() {
            pk = pk.add_gx(ui);
        }
        pk
    };
    let pk1 = {
        let mut pk = C::identity().clone();
        for FiX in vss_scheme.values() {
            pk = pk.add(&FiX[0]);
        }
        pk
    };
    assert_eq!(pk0, pk1);
    let dur = start.elapsed().as_micros() - dur;
    println!("verify 2.1 elapsed {} us", dur);

    // 2.2. Verify xji
    for i in &omega_k {
        let xji_map = &everyi_received_xji_map[&i];
        C::verify_fj_at_i(*i, xji_map, &vss_scheme).unwrap();
    }
    let dur = start.elapsed().as_micros() - dur;
    println!("verify_fj_at_i elapsed {} us", dur);

    // 3. sk recovery
    use rand::seq::IteratorRandom;
    let mut omega_s: Vec<usize> = omega_k
        .iter()
        .choose_multiple(rng, ns)
        .into_iter()
        .cloned()
        .collect();
    omega_s.sort();
    println!("signers = {:?}", &omega_s);
    let omega_s: HashSet<usize> = omega_s.into_iter().collect();
    let mut wi_map: HashMap<usize, C::ScalarT> =
        omega_s.iter().map(|i| (*i, C::zero().clone())).collect();
    for i in &omega_s {
        let wi = wi_map.get_mut(&i).unwrap();
        let lambda_i = C::lagrange_lambda(*i, &omega_s);
        assert_ne!(&lambda_i, C::zero());
        *wi = xi_map[i].mul(&lambda_i);
    }
    let sk0 = {
        let mut sk = C::zero().clone();
        for ui in ui_map.values() {
            sk = sk.add(ui);
        }
        sk
    };
    let sk1 = {
        let mut sk = C::zero().clone();
        for wi in wi_map.values() {
            sk = sk.add(wi);
        }
        sk
    };
    assert_eq!(sk0, sk1);
    let dur = start.elapsed().as_micros() - dur;
    println!("sk recover elapsed {} us", dur);
}

#[test]
fn test0() {
    use svarog_secp256k1::Secp256k1;

    // Keep parameters fixed; randomize omega_s.
    let th = 6;
    let nk = 11;
    let omega_k: HashSet<usize> = (1..=nk).collect();

    let mut rng = rand::rng();

    // 1. Keygen (same structure as `test_vss_impl`).
    let ui_map: HashMap<usize, <Secp256k1 as TrCurve>::ScalarT> = (1..=nk)
        .map(|k| (k, <Secp256k1 as TrCurve>::ScalarT::new_rand()))
        .collect();

    let mut xi_map: HashMap<usize, <Secp256k1 as TrCurve>::ScalarT> =
        (1..=nk).map(|k| (k, Secp256k1::zero().clone())).collect();
    for i in &omega_k {
        let ui = &ui_map[i];
        let (_fi_x, fij_map) = Secp256k1::generate_shares(ui, &omega_k, th);
        for (j, fij) in fij_map {
            let xj = xi_map.get_mut(&j).unwrap();
            *xj = xj.add(&fij);
        }
    }

    // 2. Re-sample omega_s and recover for 20 rounds.
    use rand::seq::IteratorRandom;
    for round in 1..=20 {
        let ns = rng.random_range(th..=nk);
        let omega_s: HashSet<usize> = omega_k
            .iter()
            .choose_multiple(&mut rng, ns)
            .into_iter()
            .cloned()
            .collect();
        assert!(omega_s.len() >= 6);

        // Compare:
        // - scalar_sum = sum( wi * lambda_i ) in the scalar field
        // - wi_int_sum = sum( wi_int ) in Z
        let mut scalar_sum = Secp256k1::zero().clone();
        let mut plan1 = Integer::from(0); // 转整数
        let mut plan2 = Integer::from(0);
        for &i in &omega_s {
            let lmdi = Secp256k1::lagrange_lambda(i, &omega_s);
            let wi = lmdi.mul(&xi_map[&i]);

            scalar_sum = scalar_sum.add(&wi);
            let wi_int = wi.to_int();
            plan1 += &wi_int;
            if wi_int > (Secp256k1::curve_order().clone() >> 1) {
                plan2 += wi_int - Secp256k1::curve_order();
            } else {
                plan2 += wi_int;
            }
        }
        assert!(scalar_sum == Scalar::new_from_int(plan1.clone()));

        println!("真私钥: \n{}", scalar_sum.to_int());
        println!(
            "\x1b[31m方案1, 绕N {} 圈: \n{}\x1b[0m",
            plan1.clone().div_euc(Secp256k1::curve_order()),
            plan1
        );
        println!(
            "\x1b[32m方案2, 绕N {} 圈: \n{}\x1b[0m",
            plan2.clone().div_euc(Secp256k1::curve_order()),
            plan2
        );
        println!("===== 第 {} 轮结束 =====", round)
    }
}