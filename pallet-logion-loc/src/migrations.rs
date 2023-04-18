use frame_support::codec::{Decode, Encode};
use frame_support::traits::Get;
use frame_support::dispatch::Vec;
use frame_support::weights::Weight;
use frame_support::traits::OnRuntimeUpgrade;

use crate::{Config, LegalOfficerCaseOf, PalletStorageVersion, pallet::StorageVersion};

pub mod v11 {
    use super::*;
    use crate::*;

    #[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, TypeInfo)]
    pub struct MetadataItemV10<AccountId> {
        name: Vec<u8>,
        value: Vec<u8>,
        submitter: AccountId,
    }

    type MetadataItemV10Of<T> = MetadataItemV10<<T as frame_system::Config>::AccountId>;

    #[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
    struct FileV10<Hash, AccountId> {
        hash: Hash,
        nature: Vec<u8>,
        submitter: AccountId,
        size: u32,
    }

    type FileV10Of<T> = FileV10<<T as pallet::Config>::Hash, <T as frame_system::Config>::AccountId>;

    #[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, TypeInfo)]
    pub struct LegalOfficerCaseV10<AccountId, Hash, LocId, BlockNumber, EthereumAddress> {
        owner: AccountId,
        requester: Requester<AccountId, LocId, EthereumAddress>,
        metadata: Vec<MetadataItemV10<AccountId>>,
        files: Vec<FileV10<Hash, AccountId>>,
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

    type LegalOfficerCaseV10Of<T> = LegalOfficerCaseV10<
        <T as frame_system::Config>::AccountId,
        <T as pallet::Config>::Hash,
        <T as pallet::Config>::LocId,
        <T as frame_system::Config>::BlockNumber,
        <T as pallet::Config>::EthereumAddress,
    >;

    pub struct EnableEthereumSubmitter<T>(sp_std::marker::PhantomData<T>);

    impl<T: Config> OnRuntimeUpgrade for EnableEthereumSubmitter<T> {
        fn on_runtime_upgrade() -> Weight {
            super::do_storage_upgrade::<T, _>(
                StorageVersion::V10AddLocFileSize,
                StorageVersion::V11EnableEthereumSubmitter,
                "EnableEthereumSubmitter",
                || {
                    LocMap::<T>::translate_values(|loc: LegalOfficerCaseV10Of<T>| {
                        let files: Vec<File<<T as pallet::Config>::Hash, <T as frame_system::Config>::AccountId, T::EthereumAddress>> = loc.files
                            .iter()
                            .map(|file: &FileV10Of<T>| File {
                                hash: file.hash,
                                nature: file.nature.clone(),
                                submitter: SupportedAccountId::Polkadot(file.submitter.clone()),
                                size: file.size,
                            })
                            .collect();
                        let metadata: Vec<MetadataItem<<T as frame_system::Config>::AccountId, T::EthereumAddress>> = loc.metadata
                            .iter()
                            .map(|item: &MetadataItemV10Of<T>| MetadataItem {
                                name: item.name.clone(),
                                value: item.value.clone(),
                                submitter: SupportedAccountId::Polkadot(item.submitter.clone()),
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
