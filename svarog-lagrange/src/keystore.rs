#![allow(nonstandard_style)]
use std::collections::HashMap;

use curve_abstract::{TrCurve, TrPoint};
use rug::Integer;
use serde::{Deserialize, Serialize};

#[allow(type_alias_bounds)]
pub type ShamirScheme<C: TrCurve> = HashMap<usize, Vec<C::PointT>>;

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Keystore<C: TrCurve> {
    pub i: usize,
    pub ui: Integer,
    pub xi: C::ScalarT,
    pub vss_scheme: ShamirScheme<C>,
    pub chain_code: [u8; 32],
    pub aux: Vec<u8>,
}

impl<C: TrCurve + 'static> Keystore<C> {
    pub fn public_key(&self) -> C::PointT {
        let gui_list: Vec<&C::PointT> = self.vss_scheme.iter().map(|(_, FjX)| &FjX[0]).collect();
        C::PointT::sum(&gui_list)
    }
}

    