use frame_support::codec::{Decode, Encode};
use frame_support::traits::Get;
use frame_support::dispatch::Vec;
use frame_support::weights::Weight;
use frame_support::traits::OnRuntimeUpgrade;

use crate::{Config, LegalOfficerCaseOf, PalletStorageVersion, pallet::StorageVersion};

pub mod v13 {
    use super::*;
    use crate::*;

    #[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, TypeInfo)]
    pub struct MetadataItemV12<AccountId, EthereumAddress> {
        name: Vec<u8>,
        value: Vec<u8>,
        submitter: SupportedAccountId<AccountId, EthereumAddress>,
    }

    type MetadataItemV12Of<T> = MetadataItemV12<<T as frame_system::Config>::AccountId, <T as pallet::Config>::EthereumAddress>;

    #[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, TypeInfo)]
    struct FileV12<Hash, AccountId, EthereumAddress> {
        hash: Hash,
        nature: Vec<u8>,
        submitter: SupportedAccountId<AccountId, EthereumAddress>,
        size: u32,
    }

    type FileV12Of<T> = FileV12<<T as pallet::Config>::Hash, <T as frame_system::Config>::AccountId, <T as pallet::Config>::EthereumAddress>;

    #[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, TypeInfo)]
    pub struct LegalOfficerCaseV12<AccountId, Hash, LocId, BlockNumber, EthereumAddress, SponsorshipId> {
        owner: AccountId,
        requester: Requester<AccountId, LocId, EthereumAddress>,
        metadata: Vec<MetadataItemV12<AccountId, EthereumAddress>>,
        files: Vec<FileV12<Hash, AccountId, EthereumAddress>>,
        closed: bool,
        loc_type: LocType,
        links: Vec<LocLink<LocId>>,
        void_info: Option<LocVoidInfo<LocId>>,
        replacer_of: Option<LocId>,
        collection_last_block_submission: Option<BlockNumber>,
        collection_max_size: Option<CollectionSize>,
        collection_can_upload: bool,
        seal: Option<Hash>,
        sponsorship_id: Option<SponsorshipId>,
    }

    type LegalOfficerCaseV12Of<T> = LegalOfficerCaseV12<
        <T as frame_system::Config>::AccountId,
        <T as pallet::Config>::Hash,
        <T as pallet::Config>::LocId,
        <T as frame_system::Config>::BlockNumber,
        <T as pallet::Config>::EthereumAddress,
        <T as pallet::Config>::SponsorshipId,
    >;

    pub struct AddAcknowledgeItems<T>(sp_std::marker::PhantomData<T>);

    impl<T: Config> OnRuntimeUpgrade for AddAcknowledgeItems<T> {
        fn on_runtime_upgrade() -> Weight {
            super::do_storage_upgrade::<T, _>(
                StorageVersion::V12Sponsorship,
                StorageVersion::V13AcknowledgeItems,
                "AddAcknowledgeItems",
                || {
                    LocMap::<T>::translate_values(|loc: LegalOfficerCaseV12Of<T>| {
                        let files: Vec<File<<T as pallet::Config>::Hash, <T as frame_system::Config>::AccountId, T::EthereumAddress>> = loc.files
                            .iter()
                            .map(|file: &FileV12Of<T>| File {
                                hash: file.hash,
                                nature: file.nature.clone(),
                                submitter: file.submitter.clone(),
                                size: file.size,
                                acknowledged: true,
                            })
                            .collect();
                        let metadata: Vec<MetadataItem<<T as frame_system::Config>::AccountId, T::EthereumAddress>> = loc.metadata
                            .iter()
                            .map(|item: &MetadataItemV12Of<T>| MetadataItem {
                                name: item.name.clone(),
                                value: item.value.clone(),
                                submitter: item.submitter.clone(),
                                acknowledged: true,
                            })
                            .collect();
                        Some(LegalOfficerCaseOf::<T> {
                            owner: loc.owner,
                            requester: loc.requester,
                            metadata,
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
                            sponsorship_id: None,
                        })
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
