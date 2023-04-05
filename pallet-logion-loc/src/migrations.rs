use frame_support::codec::{Decode, Encode};
use frame_support::traits::Get;
use frame_support::dispatch::Vec;
use frame_support::weights::Weight;
use frame_support::traits::OnRuntimeUpgrade;

use crate::{Config, LegalOfficerCaseOf, pallet, PalletStorageVersion, pallet::StorageVersion};

pub mod v10 {
    use super::*;
    use crate::*;

    #[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
    struct FileV9<Hash, AccountId> {
        hash: Hash,
        nature: Vec<u8>,
        submitter: AccountId,
    }

    type FileV9Of<T> = FileV9<<T as pallet::Config>::Hash, <T as frame_system::Config>::AccountId>;

    #[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, TypeInfo)]
    pub struct LegalOfficerCaseV9<AccountId, Hash, LocId, BlockNumber, EthereumAddress> {
        owner: AccountId,
        requester: Requester<AccountId, LocId, EthereumAddress>,
        metadata: Vec<MetadataItem<AccountId>>,
        files: Vec<FileV9<Hash, AccountId>>,
        closed: bool,
        loc_type: LocType,
        links: Vec<LocLink<LocId>>,
        void_info: Option<LocVoidInfo<LocId>>,
        replacer_of: Option<LocId>,
        collection_last_block_submission: Option<BlockNumber>,
        collection_max_size: Option<CollectionSize>,
        collection_can_upload: bool,
        seal: Option<Hash>,
    }

    type LegalOfficerCaseOfV9<T> = LegalOfficerCaseV9<
        <T as frame_system::Config>::AccountId,
        <T as pallet::Config>::Hash,
        <T as pallet::Config>::LocId,
        <T as frame_system::Config>::BlockNumber,
        <T as pallet::Config>::EthereumAddress,
    >;

    pub struct AddSizeToLocFile<T>(sp_std::marker::PhantomData<T>);

    impl<T: Config> OnRuntimeUpgrade for AddSizeToLocFile<T> {
        fn on_runtime_upgrade() -> Weight {
            super::do_storage_upgrade::<T, _>(
                StorageVersion::V9TermsAndConditions,
                StorageVersion::V10AddLocFileSize,
                "AddSizeToLocFile",
                || {
                    LocMap::<T>::translate_values(|loc: LegalOfficerCaseOfV9<T>| {
                        let files: Vec<File<<T as pallet::Config>::Hash, <T as frame_system::Config>::AccountId>> = loc.files
                            .iter()
                            .map(|file: &FileV9Of<T>| File {
                                hash: file.hash,
                                nature: file.nature.clone(),
                                submitter: file.submitter.clone(),
                                size: 0
                            })
                            .collect();
                        Some(LegalOfficerCaseOf::<T> {
                            owner: loc.owner,
                            requester: loc.requester,
                            metadata: loc.metadata,
                            files,
                            closed: loc.closed,
                            loc_type: loc.loc_type,
                            links: loc.links,
                            void_info: loc.void_info,
                            replacer_of: loc.replacer_of,
                            collection_last_block_submission: loc.collection_last_block_submission,
                            collection_max_size: loc.collection_max_size,
                            collection_can_upload: loc.collection_can_upload,
                            seal: loc.seal,
                        })
                    })
                }
            )
        }
    }
}

pub mod v9 {
    use super::*;
    use crate::{CollectionItemFile, CollectionItemsMap, CollectionItemOf, CollectionItemToken};

    #[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
    struct CollectionItemV8<Hash> {
        description: Vec<u8>,
        files: Vec<CollectionItemFile<Hash>>,
        token: Option<CollectionItemToken>,
        restricted_delivery: bool,
    }

    type CollectionItemV8Of<T> = CollectionItemV8<<T as pallet::Config>::Hash>;

    pub struct AddTermsAndConditionsToCollectionItem<T>(sp_std::marker::PhantomData<T>);
    impl<T: Config> OnRuntimeUpgrade for AddTermsAndConditionsToCollectionItem<T> {

        fn on_runtime_upgrade() -> Weight {
            super::do_storage_upgrade::<T, _>(
                StorageVersion::V8AddSeal,
                StorageVersion::V9TermsAndConditions,
                "AddTermsAndConditionsToCollectionItem",
                || {
                    CollectionItemsMap::<T>::translate(|_loc_id: T::LocId, _item_id: T::CollectionItemId, item: CollectionItemV8Of<T>| {
                        let new_item = CollectionItemOf::<T> {
                            description: item.description.clone(),
                            files: item.files.clone(),
                            token: item.token.clone(),
                            restricted_delivery: item.restricted_delivery.clone(),
                            terms_and_conditions: Vec::new(),
                        };
                        Some(new_item)
                    });
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
