use frame_support::codec::{Decode, Encode};
use frame_support::traits::Get;
use frame_support::dispatch::Vec;
use frame_support::weights::Weight;
use frame_support::traits::OnRuntimeUpgrade;

use crate::{Config, PalletStorageVersion, pallet::StorageVersion};
use super::*;


pub mod v17 {
    use super::*;
    use crate::*;

    #[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, TypeInfo)]
    pub struct CollectionItemV16<Hash, LocId, TokenIssuance> {
        description: Vec<u8>,
        files: Vec<CollectionItemFileV16<Hash>>,
        token: Option<CollectionItemTokenV16<TokenIssuance>>,
        restricted_delivery: bool,
        terms_and_conditions: Vec<TermsAndConditionsElementV16<LocId>>,
    }

    pub type CollectionItemV16Of<T> = CollectionItemV16<
        <T as pallet::Config>::Hash,
        <T as pallet::Config>::LocId,
        <T as pallet::Config>::TokenIssuance,
    >;

    #[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, TypeInfo)]
    pub struct CollectionItemFileV16<Hash> {
        name: Vec<u8>,
        content_type: Vec<u8>,
        size: u32,
        hash: Hash,
    }

    pub type CollectionItemFileV16Of<T> = CollectionItemFileV16<<T as pallet::Config>::Hash>;

    #[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, TypeInfo)]
    pub struct CollectionItemTokenV16<TokenIssuance> {
        token_type: Vec<u8>,
        token_id: Vec<u8>,
        token_issuance: TokenIssuance,
    }

    #[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, TypeInfo)]
    pub struct TermsAndConditionsElementV16<LocId> {
        tc_type: Vec<u8>,
        tc_loc: LocId,
        details: Vec<u8>,
    }

    #[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, TypeInfo)]
    pub struct TokensRecordV16<BoundedDescription, BoundedTokensRecordFilesList, AccountId> {
        description: BoundedDescription,
        files: BoundedTokensRecordFilesList,
        submitter: AccountId,
    }

    pub type TokensRecordV16Of<T> = TokensRecordV16<
        BoundedVec<u8, <T as pallet::Config>::MaxTokensRecordDescriptionSize>,
        BoundedVec<TokensRecordFileV16Of<T>, <T as pallet::Config>::MaxTokensRecordFiles>,
        <T as frame_system::Config>::AccountId,
    >;

    #[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, TypeInfo)]
    pub struct TokensRecordFileV16<Hash, BoundedName, BoundedContentType> {
        name: BoundedName,
        content_type: BoundedContentType,
        size: u32,
        hash: Hash,
    }

    pub type TokensRecordFileV16Of<T> = TokensRecordFileV16<
        <T as pallet::Config>::Hash,
        BoundedVec<u8, <T as pallet::Config>::MaxFileNameSize>,
        BoundedVec<u8, <T as pallet::Config>::MaxFileContentTypeSize>,
    >;

    pub type UnboundedTokensRecordFileV16Of<T> = TokensRecordFileV16<
        <T as pallet::Config>::Hash,
        Vec<u8>,
        Vec<u8>,
    >;

    pub struct HashItemRecordPublicData<T>(sp_std::marker::PhantomData<T>);

    impl<T: Config> OnRuntimeUpgrade for HashItemRecordPublicData<T> {
        fn on_runtime_upgrade() -> Weight {
            super::do_storage_upgrade::<T, _>(
                StorageVersion::V16MoveTokenIssuance,
                StorageVersion::V17HashItemRecordPublicData,
                "HashItemRecordPublicData",
                || {
                    CollectionItemsMap::<T>::translate_values(|item: CollectionItemV16Of<T>| {
                        let description = T::Hasher::hash(&item.description).into();

                        let files = item.files.iter()
                            .map(|file| {
                                CollectionItemFile {
                                    name: T::Hasher::hash(&file.name).into(),
                                    content_type: T::Hasher::hash(&file.content_type).into(),
                                    size: file.size,
                                    hash: file.hash,
                                }
                            })
                            .collect();

                        let terms_and_conditions = item.terms_and_conditions.iter()
                            .map(|terms| {
                                TermsAndConditionsElement {
                                    details: T::Hasher::hash(&terms.details).into(),
                                    tc_loc: terms.tc_loc,
                                    tc_type: T::Hasher::hash(&terms.tc_type).into(),
                                }
                            })
                            .collect();

                        let token = item.token.map(|some_token| {
                            CollectionItemToken {
                                token_id: T::Hasher::hash(&some_token.token_id).into(),
                                token_issuance: some_token.token_issuance,
                                token_type: T::Hasher::hash(&some_token.token_type).into(),
                            }
                        });

                        Some(CollectionItem {
                            description,
                            files,
                            restricted_delivery: item.restricted_delivery,
                            terms_and_conditions,
                            token,
                        })
                    });

                    TokensRecordsMap::<T>::translate_values(|record: TokensRecordV16Of<T>| {
                        let description = T::Hasher::hash(&record.description).into();

                        let mut files: BoundedVec<TokensRecordFileOf<T>, T::MaxTokensRecordFiles> = BoundedVec::with_bounded_capacity(record.files.len());
                        for file in record.files.iter() {
                            files.try_push(TokensRecordFile {
                                name: T::Hasher::hash(&file.name).into(),
                                content_type: T::Hasher::hash(&file.content_type).into(),
                                size: file.size,
                                hash: file.hash,
                            }).unwrap();
                        }

                        Some(TokensRecord {
                            description,
                            files,
                            submitter: record.submitter,
                        })
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
