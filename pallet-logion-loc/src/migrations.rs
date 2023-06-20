use frame_support::codec::{Decode, Encode};
use frame_support::traits::Get;
use frame_support::dispatch::Vec;
use frame_support::weights::Weight;
use frame_support::traits::OnRuntimeUpgrade;

use crate::{Config, PalletStorageVersion, pallet::StorageVersion};

pub mod v15 {
    use super::*;
    use crate::*;

    #[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, TypeInfo)]
    struct CollectionItemV14<Hash, LocId> {
        description: Vec<u8>,
        files: Vec<CollectionItemFile<Hash>>,
        token: Option<CollectionItemToken>,
        restricted_delivery: bool,
        terms_and_conditions: Vec<TermsAndConditionsElement<LocId>>,
    }

    type CollectionItemV14Of<T> = CollectionItemV14<
        <T as pallet::Config>::Hash,
        <T as pallet::Config>::LocId,
    >;

    pub struct AddTokenIssuance<T>(sp_std::marker::PhantomData<T>);

    impl<T: Config> OnRuntimeUpgrade for AddTokenIssuance<T> {
        fn on_runtime_upgrade() -> Weight {
            super::do_storage_upgrade::<T, _>(
                StorageVersion::V14HashLocPublicData,
                StorageVersion::V15AddTokenIssuance,
                "AddTokenIssuance",
                || {
                    CollectionItemsMap::<T>::translate_values(|item: CollectionItemV14Of<T>| {
                        Some(CollectionItemOf::<T> {
                            description: item.description,
                            files: item.files,
                            token: item.token,
                            restricted_delivery: item.restricted_delivery,
                            terms_and_conditions: item.terms_and_conditions,
                            token_issuance: 0u32.into(),
                        })
                    })
                }
            )
        }
    }
}

pub mod v14 {
    use super::*;
    use crate::*;

    #[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, TypeInfo)]
    pub struct MetadataItemV13<AccountId, EthereumAddress> {
        name: Vec<u8>,
        value: Vec<u8>,
        submitter: SupportedAccountId<AccountId, EthereumAddress>,
        acknowledged: bool,
    }

    type MetadataItemV13Of<T> = MetadataItemV13<<T as frame_system::Config>::AccountId, <T as pallet::Config>::EthereumAddress>;

    #[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, TypeInfo)]
    struct FileV13<Hash, AccountId, EthereumAddress> {
        hash: Hash,
        nature: Vec<u8>,
        submitter: SupportedAccountId<AccountId, EthereumAddress>,
        size: u32,
        acknowledged: bool,
    }

    type FileV13Of<T> = FileV13<<T as pallet::Config>::Hash, <T as frame_system::Config>::AccountId, <T as pallet::Config>::EthereumAddress>;

    #[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, TypeInfo)]
    pub struct LocLinkV13<LocId> {
        id: LocId,
        nature: Vec<u8>,
    }

    type LocLinkV13Of<T> = LocLinkV13<<T as pallet::Config>::LocId>;

    #[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, TypeInfo)]
    pub struct LegalOfficerCaseV13<AccountId, Hash, LocId, BlockNumber, EthereumAddress, SponsorshipId> {
        owner: AccountId,
        requester: Requester<AccountId, LocId, EthereumAddress>,
        metadata: Vec<MetadataItemV13<AccountId, EthereumAddress>>,
        files: Vec<FileV13<Hash, AccountId, EthereumAddress>>,
        closed: bool,
        loc_type: LocType,
        links: Vec<LocLinkV13<LocId>>,
        void_info: Option<LocVoidInfo<LocId>>,
        replacer_of: Option<LocId>,
        collection_last_block_submission: Option<BlockNumber>,
        collection_max_size: Option<CollectionSize>,
        collection_can_upload: bool,
        seal: Option<Hash>,
        sponsorship_id: Option<SponsorshipId>,
    }

    type LegalOfficerCaseV13Of<T> = LegalOfficerCaseV13<
        <T as frame_system::Config>::AccountId,
        <T as pallet::Config>::Hash,
        <T as pallet::Config>::LocId,
        <T as frame_system::Config>::BlockNumber,
        <T as pallet::Config>::EthereumAddress,
        <T as pallet::Config>::SponsorshipId,
    >;

    pub struct HashLocPublicData<T>(sp_std::marker::PhantomData<T>);

    impl<T: Config> OnRuntimeUpgrade for HashLocPublicData<T> {
        fn on_runtime_upgrade() -> Weight {
            super::do_storage_upgrade::<T, _>(
                StorageVersion::V13AcknowledgeItems,
                StorageVersion::V14HashLocPublicData,
                "HashLocPublicData",
                || {
                    LocMap::<T>::translate_values(|loc: LegalOfficerCaseV13Of<T>| {
                        let files: Vec<File<<T as pallet::Config>::Hash, <T as frame_system::Config>::AccountId, T::EthereumAddress>> = loc.files
                            .iter()
                            .map(|file: &FileV13Of<T>| File {
                                hash: file.hash,
                                nature: T::Hasher::hash(&file.nature).into(),
                                submitter: file.submitter.clone(),
                                size: file.size,
                                acknowledged: file.acknowledged,
                            })
                            .collect();
                        let metadata: Vec<MetadataItem<<T as frame_system::Config>::AccountId, T::EthereumAddress, <T as pallet::Config>::Hash>> = loc.metadata
                            .iter()
                            .map(|item: &MetadataItemV13Of<T>| MetadataItem {
                                name: T::Hasher::hash(&item.name),
                                value: T::Hasher::hash(&item.value),
                                submitter: item.submitter.clone(),
                                acknowledged: item.acknowledged,
                            })
                            .collect();
                        let links: Vec<LocLink<<T as pallet::Config>::LocId, <T as pallet::Config>::Hash>> = loc.links
                            .iter()
                            .map(|link: &LocLinkV13Of<T>| LocLink {
                                id: link.id.clone(),
                                nature: T::Hasher::hash(&link.nature),
                            })
                            .collect();
                        Some(LegalOfficerCaseOf::<T> {
                            owner: loc.owner,
                            requester: loc.requester,
                            metadata,
                            files,
                            closed: loc.closed,
                            loc_type: loc.loc_type,
                            links,
                            void_info: loc.void_info,
                            replacer_of: loc.replacer_of,
                            collection_last_block_submission: loc.collection_last_block_submission,
                            collection_max_size: loc.collection_max_size,
                            collection_can_upload: loc.collection_can_upload,
                            seal: loc.seal,
                            sponsorship_id: loc.sponsorship_id,
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
