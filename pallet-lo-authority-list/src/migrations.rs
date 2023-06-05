use frame_support::traits::Get;
use frame_support::weights::Weight;
use frame_support::traits::OnRuntimeUpgrade;

use crate::{Config, PalletStorageVersion, pallet::StorageVersion};

pub mod v4 {
    use super::*;
    use crate::*;

    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
    pub enum LegalOfficerDataV3<AccountId> {
        Host(HostDataV3),
        Guest(AccountId),
    }

    pub type LegalOfficerDataV3Of<T> = LegalOfficerDataV3<
        <T as frame_system::Config>::AccountId,
    >;

    #[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
    pub struct HostDataV3 {
        pub node_id: Option<PeerId>,
        pub base_url: Option<Vec<u8>>,
    }

    pub struct AddRegion<T>(sp_std::marker::PhantomData<T>);
    impl<T: Config> OnRuntimeUpgrade for AddRegion<T> {

        fn on_runtime_upgrade() -> Weight {
            super::do_storage_upgrade::<T, _>(
                StorageVersion::V3GuestLegalOfficers,
                StorageVersion::V4Region,
                "AddRegion",
                || {
                    LegalOfficerSet::<T>::translate_values(|legal_officer_data: LegalOfficerDataV3Of<T>| {
                        match legal_officer_data {
                            LegalOfficerDataV3::Guest(guest_data) => Some(LegalOfficerData::Guest(guest_data)),
                            LegalOfficerDataV3::Host(host_data) => Some(LegalOfficerData::Host(HostData {
                                node_id: host_data.node_id.clone(),
                                base_url: host_data.base_url.clone(),
                                region: Default::default(),
                            })),
                        }
                    })
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
