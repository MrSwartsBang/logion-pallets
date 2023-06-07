#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_logion_loc.
pub trait WeightInfo {
    fn create_vote_for_all_legal_officers() -> Weight;
    fn vote() -> Weight;
}

/// Weights for pallet_logion_loc using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
    fn create_vote_for_all_legal_officers() -> Weight {
        Weight::from_parts(29_862_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn vote() -> Weight {
        Weight::from_parts(20_945_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}

// For backwards compatibility and tests
impl WeightInfo for () {
    fn create_vote_for_all_legal_officers() -> Weight {
        Weight::from_parts(29_862_000, 0)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
    fn vote() -> Weight {
        Weight::from_parts(20_945_000, 0)
            .saturating_add(RocksDbWeight::get().reads(1))
            .saturating_add(RocksDbWeight::get().writes(1))
    }
}
