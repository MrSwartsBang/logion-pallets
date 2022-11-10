use frame_support::traits::Get;
use frame_support::weights::Weight;
use frame_support::traits::OnRuntimeUpgrade;
use sp_std::collections::btree_set::BTreeSet;

use crate::{Config, PalletStorageVersion, pallet::StorageVersion};

pub mod v2 {
	use super::*;
	use crate::{LegalOfficerSet, LegalOfficerNodes};

	pub struct AddOnchainSettings<T>(sp_std::marker::PhantomData<T>);
	impl<T: Config> OnRuntimeUpgrade for AddOnchainSettings<T> {

		fn on_runtime_upgrade() -> Weight {
			super::do_storage_upgrade::<T, _>(
				StorageVersion::V1,
				StorageVersion::V2AddOnchainSettings,
				"AddOnchainSettings",
				|| {
					LegalOfficerSet::<T>::translate(|_, _: bool| Some(Default::default()));
					LegalOfficerNodes::<T>::set(BTreeSet::new());
				}
			)
		}
	}
}

fn do_storage_upgrade<T: Config, F>(expected_version: StorageVersion, target_version: StorageVersion, migration_name: &str, migration: F) -> Weight
where F: FnOnce() -> () {
	let storage_version = PalletStorageVersion::<T>::get();
	if storage_version == expected_version {
		migration();

		PalletStorageVersion::<T>::set(target_version);
		log::info!("✅ {:?} migration successfully executed", migration_name);
		T::BlockWeights::get().max_block
	} else {
		if storage_version != target_version {
			log::warn!("❗ {:?} cannot run migration with storage version {:?} (expected {:?})", migration_name, storage_version, expected_version);
		} else {
			log::info!("❎ {:?} execution skipped, already at target version {:?}", migration_name, target_version);
		}
		T::DbWeight::get().reads(1)
	}
}
