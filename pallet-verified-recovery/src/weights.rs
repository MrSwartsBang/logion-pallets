#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_verified_recovery.
pub trait WeightInfo {
	fn create_recovery() -> Weight;
}

/// Default weights
pub struct DefaultWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for DefaultWeight<T> {
	fn create_recovery() -> Weight {
		Weight::from_ref_time(33_904_000)
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
}

impl WeightInfo for () {
	fn create_recovery() -> Weight {
		Weight::from_ref_time(33_904_000)
			.saturating_add(RocksDbWeight::get().reads(2))
			.saturating_add(RocksDbWeight::get().writes(1))
	}
}
