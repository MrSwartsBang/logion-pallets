use core::str::FromStr;
use frame_support::{assert_err, assert_ok};
use frame_support::error::BadOrigin;
use frame_support::traits::Len;
use sp_core::{H256, H160};
use sp_runtime::traits::BlakeTwo256;
use sp_runtime::traits::Hash;

use logion_shared::{Beneficiary, LocQuery, LocValidity};

use crate::TokensRecordFileOf;
use crate::{
    Error, File, LegalOfficerCase, LocLink, LocType, MetadataItem, CollectionItem, CollectionItemFile,
    CollectionItemToken, mock::*, TermsAndConditionsElement, TokensRecordFile,
    VerifiedIssuer, OtherAccountId, SupportedAccountId, MetadataItemParams, FileParams, Hasher,
    Requester::{Account, OtherAccount}, fees::*,
};

const LOC_ID: u32 = 0;
const OTHER_LOC_ID: u32 = 1;
const LOGION_CLASSIFICATION_LOC_ID: u32 = 2;
const ADDITIONAL_TC_LOC_ID: u32 = 3;
const ISSUER1_IDENTITY_LOC_ID: u32 = 4;
const ISSUER2_IDENTITY_LOC_ID: u32 = 5;
const FILE_SIZE: u32 = 90;
const ONE_LGNT: Balance = 1_000_000_000_000_000_000;
const INITIAL_BALANCE: Balance = (3 * 2000 * ONE_LGNT) + ONE_LGNT;
const INSUFFICIENT_BALANCE: Balance = 99;
const ACKNOWLEDGED: bool = true;
const NOT_ACKNOWLEDGED: bool = !ACKNOWLEDGED;

#[test]
fn it_creates_loc() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        let snapshot = BalancesSnapshot::take(LOC_REQUESTER_ID, LOC_OWNER1);
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        assert_eq!(LogionLoc::loc(LOC_ID), Some(LegalOfficerCase {
            owner: LOC_OWNER1,
            requester: LOC_REQUESTER,
            metadata: vec![],
            files: vec![],
            closed: false,
            loc_type: LocType::Transaction,
            links: vec![],
            void_info: None,
            replacer_of: None,
            collection_last_block_submission: None,
            collection_max_size: None,
            collection_can_upload: false,
            seal: None,
            sponsorship_id: None,
        }));

        let fees = Fees::only_legal(2000 * ONE_LGNT, Beneficiary::LegalOfficer(LOC_OWNER1));
        fees.assert_balances_events(snapshot);
    });
}

fn setup_default_balances() {
    set_balance(LOC_REQUESTER_ID, INITIAL_BALANCE);
    set_balance(SPONSOR_ID, INITIAL_BALANCE);
    set_balance(LOC_OWNER1, INITIAL_BALANCE);
    set_balance(LOC_OWNER2, INITIAL_BALANCE);
    set_balance(ISSUER_ID1, INITIAL_BALANCE);
    set_balance(ISSUER_ID2, INITIAL_BALANCE);
    set_balance(TREASURY_ACCOUNT_ID, INITIAL_BALANCE);
}

fn set_balance(account_id: AccountId, amount: Balance) {
    assert_ok!(Balances::force_set_balance(RuntimeOrigin::root(), account_id, amount));
}

#[test]
fn it_makes_existing_loc_void() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        assert_ok!(LogionLoc::make_void(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

        let void_info = LogionLoc::loc(LOC_ID).unwrap().void_info;
        assert!(void_info.is_some());
        assert!(!void_info.unwrap().replacer.is_some());
    });
}

#[test]
fn it_makes_existing_loc_void_and_replace_it() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        create_closed_loc();

        const REPLACER_LOC_ID: u32 = OTHER_LOC_ID;
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), REPLACER_LOC_ID, LOC_OWNER1));

        assert_ok!(LogionLoc::make_void_and_replace(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, REPLACER_LOC_ID));

        let void_info = LogionLoc::loc(LOC_ID).unwrap().void_info;
        assert!(void_info.is_some());
        let replacer: Option<u32> = void_info.unwrap().replacer;
        assert!(replacer.is_some());
        assert_eq!(replacer.unwrap(), REPLACER_LOC_ID);

        let replacer_loc = LogionLoc::loc(REPLACER_LOC_ID).unwrap();
        assert!(replacer_loc.replacer_of.is_some());
        assert_eq!(replacer_loc.replacer_of.unwrap(), LOC_ID)
    });
}

#[test]
fn it_fails_making_existing_loc_void_for_unauthorized_caller() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        assert_err!(LogionLoc::make_void(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID), Error::<Test>::Unauthorized);
        let void_info = LogionLoc::loc(LOC_ID).unwrap().void_info;
        assert!(!void_info.is_some());
    });
}

#[test]
fn it_fails_making_existing_loc_void_for_already_void_loc() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        assert_ok!(LogionLoc::make_void(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));
        assert_err!(LogionLoc::make_void(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID), Error::<Test>::AlreadyVoid);
    });
}

#[test]
fn it_fails_replacing_with_non_existent_loc() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        assert_err!(LogionLoc::make_void_and_replace(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, OTHER_LOC_ID), Error::<Test>::ReplacerLocNotFound);
    });
}

#[test]
fn it_fails_replacing_with_void_loc() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        const REPLACER_LOC_ID: u32 = OTHER_LOC_ID;
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), OTHER_LOC_ID, LOC_OWNER1));
        assert_ok!(LogionLoc::make_void(RuntimeOrigin::signed(LOC_OWNER1), OTHER_LOC_ID));
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        assert_err!(LogionLoc::make_void_and_replace(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, REPLACER_LOC_ID), Error::<Test>::ReplacerLocAlreadyVoid);
    });
}

#[test]
fn it_fails_replacing_with_loc_already_replacing_another_loc() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        const REPLACER_LOC_ID: u32 = 2;
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), OTHER_LOC_ID, LOC_OWNER1));
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), REPLACER_LOC_ID, LOC_OWNER1));
        assert_ok!(LogionLoc::make_void_and_replace(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, REPLACER_LOC_ID));
        assert_err!(LogionLoc::make_void_and_replace(RuntimeOrigin::signed(LOC_OWNER1), OTHER_LOC_ID, REPLACER_LOC_ID), Error::<Test>::ReplacerLocAlreadyReplacing);
    });
}

#[test]
fn it_fails_replacing_with_wrongly_typed_loc() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        const REPLACER_LOC_ID: u32 = 2;
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        assert_ok!(LogionLoc::create_polkadot_identity_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), REPLACER_LOC_ID, LOC_OWNER1));
        assert_err!(LogionLoc::make_void_and_replace(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, REPLACER_LOC_ID), Error::<Test>::ReplacerLocWrongType);
    });
}

#[test]
fn it_adds_metadata_when_caller_and_submitter_is_owner() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        let metadata = MetadataItemParams {
            name: sha256(&vec![1, 2, 3]),
            value: sha256(&vec![4, 5, 6]),
            submitter: SupportedAccountId::Polkadot(LOC_OWNER1),
        };
        assert_ok!(LogionLoc::add_metadata(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, metadata.clone()));
        let loc = LogionLoc::loc(LOC_ID).unwrap();
        assert_eq!(loc.metadata[0], expected_metadata(metadata, ACKNOWLEDGED));
    });
}

fn sha256(data: &Vec<u8>) -> H256 {
    <SHA256 as Hasher<H256>>::hash(data)
}

fn expected_metadata(metadata: MetadataItemParams<AccountId, EthereumAddress, crate::mock::Hash>, acknowledged: bool) -> MetadataItem<AccountId, EthereumAddress, crate::mock::Hash> {
    return MetadataItem {
        name: metadata.name,
        value: metadata.value,
        submitter: metadata.submitter,
        acknowledged,
    };
}

#[test]
fn it_adds_metadata_when_caller_is_owner_and_submitter_is_requester() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        let metadata = MetadataItemParams {
            name: sha256(&vec![1, 2, 3]),
            value: sha256(&vec![4, 5, 6]),
            submitter: SupportedAccountId::Polkadot(LOC_REQUESTER_ID),
        };
        assert_ok!(LogionLoc::add_metadata(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, metadata.clone()));
        let loc = LogionLoc::loc(LOC_ID).unwrap();
        assert_eq!(loc.metadata[0], expected_metadata(metadata, ACKNOWLEDGED));
    });
}

#[test]
fn it_adds_metadata_when_caller_is_requester() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        let metadata = MetadataItemParams {
            name: sha256(&vec![1, 2, 3]),
            value: sha256(&vec![4, 5, 6]),
            submitter: SupportedAccountId::Polkadot(LOC_REQUESTER_ID),
        };
        assert_ok!(LogionLoc::add_metadata(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, metadata.clone()));
        let loc = LogionLoc::loc(LOC_ID).unwrap();
        assert_eq!(loc.metadata[0], expected_metadata(metadata, NOT_ACKNOWLEDGED));
    });
}

#[test]
fn it_acknowledges_metadata() {
    new_test_ext().execute_with(|| {
        let metadata = create_loc_with_metadata_from_requester();
        assert_ok!(LogionLoc::acknowledge_metadata(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, metadata.name.clone()));
        let loc = LogionLoc::loc(LOC_ID).unwrap();
        assert_eq!(loc.metadata[0], expected_metadata(metadata.clone(), ACKNOWLEDGED));
    });
}

#[test]
fn it_fails_to_acknowledge_unknown_metadata() {
    new_test_ext().execute_with(|| {
        create_loc_with_metadata_from_requester();
        let name = sha256(&"unknown_metadata".as_bytes().to_vec());
        assert_err!(LogionLoc::acknowledge_metadata(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, name), Error::<Test>::ItemNotFound);
    });
}

#[test]
fn it_fails_to_acknowledge_already_acknowledged_metadata() {
    new_test_ext().execute_with(|| {
        let metadata = create_loc_with_metadata_from_requester();
        assert_ok!(LogionLoc::acknowledge_metadata(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, metadata.name.clone()));
        assert_err!(LogionLoc::acknowledge_metadata(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, metadata.name.clone()), Error::<Test>::ItemAlreadyAcknowledged);
    });
}

#[test]
fn it_fails_to_acknowledge_metadata_when_unauthorized_caller() {
    new_test_ext().execute_with(|| {
        let metadata = create_loc_with_metadata_from_requester();
        assert_err!(LogionLoc::acknowledge_metadata(RuntimeOrigin::signed(UNAUTHORIZED_CALLER), LOC_ID, metadata.name.clone()), BadOrigin);
    });
}

#[test]
fn it_fails_to_close_loc_with_unacknowledged_metadata() {
    new_test_ext().execute_with(|| {
        create_loc_with_metadata_from_requester();
        assert_err!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID), Error::<Test>::CannotCloseUnacknowledged);
    });
}

#[test]
fn it_fails_to_acknowledge_metadata_when_loc_voided() {
    new_test_ext().execute_with(|| {
        let metadata = create_loc_with_metadata_from_requester();
        assert_ok!(LogionLoc::make_void(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));
        assert_err!(LogionLoc::acknowledge_metadata(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, metadata.name.clone()), Error::<Test>::CannotMutateVoid);
    });
}

fn create_loc_with_metadata_from_requester() -> MetadataItemParams<AccountId, EthereumAddress, crate::mock::Hash> {
    setup_default_balances();
    assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
    let metadata = MetadataItemParams {
        name: sha256(&vec![1, 2, 3]),
        value: sha256(&vec![4, 5, 6]),
        submitter: SupportedAccountId::Polkadot(LOC_REQUESTER_ID),
    };
    assert_ok!(LogionLoc::add_metadata(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, metadata.clone()));
    let loc = LogionLoc::loc(LOC_ID).unwrap();
    assert_eq!(loc.metadata[0], expected_metadata(metadata.clone(), NOT_ACKNOWLEDGED));
    metadata
}

#[test]
fn it_fails_adding_metadata_for_unauthorized_caller() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        let metadata = MetadataItemParams {
            name: sha256(&vec![1, 2, 3]),
            value: sha256(&vec![4, 5, 6]),
            submitter: SupportedAccountId::Polkadot(LOC_OWNER1),
        };
        assert_err!(LogionLoc::add_metadata(RuntimeOrigin::signed(UNAUTHORIZED_CALLER), LOC_ID, metadata.clone()), Error::<Test>::Unauthorized);
    });
}

#[test]
fn it_fails_adding_metadata_when_closed() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        create_closed_loc();
        let metadata = MetadataItemParams {
            name: sha256(&vec![1, 2, 3]),
            value: sha256(&vec![4, 5, 6]),
            submitter: SupportedAccountId::Polkadot(LOC_OWNER1),
        };
        assert_err!(LogionLoc::add_metadata(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, metadata.clone()), Error::<Test>::CannotMutate);
    });
}

#[test]
fn it_fails_adding_metadata_when_invalid_submitter() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        let metadata = MetadataItemParams {
            name: sha256(&vec![1, 2, 3]),
            value: sha256(&vec![4, 5, 6]),
            submitter: SupportedAccountId::Polkadot(LOC_OWNER1),
        };
        assert_err!(LogionLoc::add_metadata(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, metadata.clone()), Error::<Test>::Unauthorized);
    });
}

fn create_closed_loc() {
    assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
    assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));
}

#[test]
fn it_adds_file_when_caller_owner_and_submitter_is_owner() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        let file = FileParams {
            hash: sha256(&"test".as_bytes().to_vec()),
            nature: sha256(&"test-file-nature".as_bytes().to_vec()),
            submitter: SupportedAccountId::Polkadot(LOC_OWNER1),
            size: FILE_SIZE,
        };
        let snapshot = BalancesSnapshot::take(LOC_REQUESTER_ID, LOC_OWNER1);
        assert_ok!(LogionLoc::add_file(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, file.clone()));
        let loc = LogionLoc::loc(LOC_ID).unwrap();
        assert_eq!(loc.files[0], expected_file(&file, ACKNOWLEDGED));

        let fees = Fees::only_storage(1, file.size);
        fees.assert_balances_events(snapshot);
    });
}

fn expected_file(file: &FileParams<H256, AccountId, EthereumAddress>, acknowledged: bool) -> File<H256, AccountId, EthereumAddress> {
    return File {
        hash: file.hash,
        nature: file.nature.clone(),
        submitter: file.submitter,
        size: file.size,
        acknowledged,
    }
}

#[test]
fn it_adds_file_when_caller_is_owner_and_submitter_is_requester() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        let file = FileParams {
            hash: sha256(&"test".as_bytes().to_vec()),
            nature: sha256(&"test-file-nature".as_bytes().to_vec()),
            submitter: SupportedAccountId::Polkadot(LOC_REQUESTER_ID),
            size: FILE_SIZE,
        };
        let snapshot = BalancesSnapshot::take(LOC_REQUESTER_ID, LOC_OWNER1);
        assert_ok!(LogionLoc::add_file(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, file.clone()));
        let loc = LogionLoc::loc(LOC_ID).unwrap();
        assert_eq!(loc.files[0], expected_file(&file, ACKNOWLEDGED));
        let fees = Fees::only_storage(1, file.size);
        fees.assert_balances_events(snapshot);
    });
}

#[test]
fn it_adds_file_when_caller_is_requester() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        let file = FileParams {
            hash: sha256(&"test".as_bytes().to_vec()),
            nature: sha256(&"test-file-nature".as_bytes().to_vec()),
            submitter: SupportedAccountId::Polkadot(LOC_REQUESTER_ID),
            size: FILE_SIZE,
        };
        let snapshot = BalancesSnapshot::take(LOC_REQUESTER_ID, LOC_OWNER1);
        assert_ok!(LogionLoc::add_file(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, file.clone()));
        let loc = LogionLoc::loc(LOC_ID).unwrap();
        assert_eq!(loc.files[0], expected_file(&file, NOT_ACKNOWLEDGED));
        let fees = Fees::only_storage(1, file.size);
        fees.assert_balances_events(snapshot);
    });
}

#[test]
fn it_acknowledges_file() {
    new_test_ext().execute_with(|| {
        let file = create_loc_with_file_from_requester();
        assert_ok!(LogionLoc::acknowledge_file(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, file.hash.clone()));
        let loc = LogionLoc::loc(LOC_ID).unwrap();
        assert_eq!(loc.files[0], expected_file(&file, ACKNOWLEDGED));
    });
}

#[test]
fn it_fails_to_acknowledge_unknown_file() {
    new_test_ext().execute_with(|| {
        create_loc_with_file_from_requester();
        let hash = BlakeTwo256::hash_of(&"unknown_hash".as_bytes().to_vec());
        assert_err!(LogionLoc::acknowledge_file(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, hash), Error::<Test>::ItemNotFound);
    });
}

#[test]
fn it_fails_to_acknowledge_already_acknowledged_file() {
    new_test_ext().execute_with(|| {
        let file = create_loc_with_file_from_requester();
        assert_ok!(LogionLoc::acknowledge_file(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, file.hash.clone()));
        assert_err!(LogionLoc::acknowledge_file(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, file.hash.clone()), Error::<Test>::ItemAlreadyAcknowledged);
    });
}

#[test]
fn it_fails_to_acknowledge_file_when_unauthorized_caller() {
    new_test_ext().execute_with(|| {
        let file = create_loc_with_file_from_requester();
        assert_err!(LogionLoc::acknowledge_file(RuntimeOrigin::signed(UNAUTHORIZED_CALLER), LOC_ID, file.hash.clone()), BadOrigin);
    });
}

#[test]
fn it_fails_to_close_loc_with_unacknowledged_file() {
    new_test_ext().execute_with(|| {
        create_loc_with_file_from_requester();
        assert_err!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID), Error::<Test>::CannotCloseUnacknowledged);
    });
}

#[test]
fn it_fails_to_acknowledge_file_when_loc_voided() {
    new_test_ext().execute_with(|| {
        let file = create_loc_with_file_from_requester();
        assert_ok!(LogionLoc::make_void(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));
        assert_err!(LogionLoc::acknowledge_file(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, file.hash.clone()), Error::<Test>::CannotMutateVoid);
    });
}

fn create_loc_with_file_from_requester() -> FileParams<H256, AccountId, EthereumAddress> {
    setup_default_balances();
    assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
    let file = FileParams {
        hash: sha256(&"test".as_bytes().to_vec()),
        nature: sha256(&"test-file-nature".as_bytes().to_vec()),
        submitter: SupportedAccountId::Polkadot(LOC_REQUESTER_ID),
        size: FILE_SIZE,
    };
    assert_ok!(LogionLoc::add_file(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, file.clone()));
    let loc = LogionLoc::loc(LOC_ID).unwrap();
    assert_eq!(loc.files[0], expected_file(&file, NOT_ACKNOWLEDGED));
    file
}

#[test]
fn it_fails_adding_file_for_unauthorized_caller() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        let file = FileParams {
            hash: sha256(&"test".as_bytes().to_vec()),
            nature: sha256(&"test-file-nature".as_bytes().to_vec()),
            submitter: SupportedAccountId::Polkadot(LOC_OWNER1),
            size: FILE_SIZE,
        };
        assert_err!(LogionLoc::add_file(RuntimeOrigin::signed(UNAUTHORIZED_CALLER), LOC_ID, file.clone()), Error::<Test>::Unauthorized);
    });
}

#[test]
fn it_fails_adding_file_when_insufficient_funds() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        set_balance(LOC_REQUESTER_ID, INSUFFICIENT_BALANCE);
        let file = FileParams {
            hash: sha256(&"test".as_bytes().to_vec()),
            nature: sha256(&"test-file-nature".as_bytes().to_vec()),
            submitter: SupportedAccountId::Polkadot(LOC_REQUESTER_ID),
            size: FILE_SIZE,
        };
        let snapshot = BalancesSnapshot::take(LOC_REQUESTER_ID, LOC_OWNER1);
        assert_err!(LogionLoc::add_file(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, file.clone()), Error::<Test>::InsufficientFunds);
        check_no_fees(snapshot);
    });
}

fn check_no_fees(previous_balances: BalancesSnapshot) {
    let current_balances = BalancesSnapshot::take(previous_balances.payer_account, previous_balances.legal_officer_account);
    let balances_delta = current_balances.delta_since(&previous_balances);

    assert_eq!(balances_delta.total_credited(), 0);
    assert_eq!(balances_delta.total_debited(), 0);
}

#[test]
fn it_fails_adding_file_when_closed() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        create_closed_loc();
        let file = FileParams {
            hash: sha256(&"test".as_bytes().to_vec()),
            nature: sha256(&"test-file-nature".as_bytes().to_vec()),
            submitter: SupportedAccountId::Polkadot(LOC_OWNER1),
            size: FILE_SIZE,
        };
        assert_err!(LogionLoc::add_file(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, file.clone()), Error::<Test>::CannotMutate);
    });
}

#[test]
fn it_adds_link() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), OTHER_LOC_ID, LOC_OWNER1));
        let link = LocLink {
            id: OTHER_LOC_ID,
            nature: sha256(&"test-link-nature".as_bytes().to_vec()),
        };
        assert_ok!(LogionLoc::add_link(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, link.clone()));
        let loc = LogionLoc::loc(LOC_ID).unwrap();
        assert_eq!(loc.links[0], link);
    });
}

#[test]
fn it_fails_adding_link_for_unauthorized_caller() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), OTHER_LOC_ID, LOC_OWNER1));
        let link = LocLink {
            id: OTHER_LOC_ID,
            nature: sha256(&"test-link-nature".as_bytes().to_vec()),
        };
        assert_err!(LogionLoc::add_link(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, link.clone()), Error::<Test>::Unauthorized);
    });
}

#[test]
fn it_fails_adding_link_when_closed() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        create_closed_loc();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), OTHER_LOC_ID, LOC_OWNER1));
        let link = LocLink {
            id: OTHER_LOC_ID,
            nature: sha256(&"test-link-nature".as_bytes().to_vec()),
        };
        assert_err!(LogionLoc::add_link(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, link.clone()), Error::<Test>::CannotMutate);
    });
}

#[test]
fn it_fails_adding_wrong_link() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        let link = LocLink {
            id: OTHER_LOC_ID,
            nature: sha256(&"test-link-nature".as_bytes().to_vec()),
        };
        assert_err!(LogionLoc::add_link(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, link.clone()), Error::<Test>::LinkedLocNotFound);
    });
}

#[test]
fn it_closes_loc() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));
        let loc = LogionLoc::loc(LOC_ID).unwrap();
        assert!(loc.closed);
        assert!(loc.seal.is_none());
    });
}

#[test]
fn it_fails_closing_loc_for_unauthorized_caller() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        assert_err!(LogionLoc::close(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID), Error::<Test>::Unauthorized);
    });
}

#[test]
fn it_fails_closing_loc_for_already_closed() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        create_closed_loc();
        assert_err!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID), Error::<Test>::AlreadyClosed);
    });
}

#[test]
fn it_links_locs_to_account() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        assert_ok!(LogionLoc::create_polkadot_identity_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), OTHER_LOC_ID, LOC_OWNER1));
        assert!(LogionLoc::account_locs(LOC_REQUESTER_ID).is_some());
        assert!(LogionLoc::account_locs(LOC_REQUESTER_ID).unwrap().len() == 2);
        assert_eq!(LogionLoc::account_locs(LOC_REQUESTER_ID).unwrap()[0], LOC_ID);
        assert_eq!(LogionLoc::account_locs(LOC_REQUESTER_ID).unwrap()[1], OTHER_LOC_ID);
    });
}

#[test]
fn it_fails_creating_loc_with_non_legal_officer() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_err!(LogionLoc::create_polkadot_identity_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_REQUESTER_ID), Error::<Test>::Unauthorized);
        assert_err!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_REQUESTER_ID), Error::<Test>::Unauthorized);
        assert_err!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_REQUESTER_ID, None, Some(10), false), Error::<Test>::Unauthorized);
    });
}

#[test]
fn it_detects_existing_identity_loc() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_identity_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

        assert_ok!(LogionLoc::create_polkadot_identity_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), OTHER_LOC_ID, LOC_OWNER2));
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER2), OTHER_LOC_ID));

        let legal_officers = Vec::from([LOC_OWNER1, LOC_OWNER2]);
        assert!(LogionLoc::has_closed_identity_locs(&LOC_REQUESTER_ID, &legal_officers));
    });
}

#[test]
fn it_detects_valid_loc_with_owner() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_identity_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));
        assert_eq!(LogionLoc::loc_valid_with_owner(&LOC_ID, &LOC_OWNER1), true);
    });
}

#[test]
fn it_detects_non_existing_loc_as_invalid() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_eq!(LogionLoc::loc_valid_with_owner(&LOC_ID, &LOC_OWNER1), false);
    });
}

#[test]
fn it_detects_open_loc_as_invalid() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_identity_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        assert_eq!(LogionLoc::loc_valid_with_owner(&LOC_ID, &LOC_OWNER1), false);
    });
}

#[test]
fn it_detects_void_loc_as_invalid() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_identity_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        assert_ok!(LogionLoc::make_void(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));
        assert_eq!(LogionLoc::loc_valid_with_owner(&LOC_ID, &LOC_OWNER1), false);
    });
}

#[test]
fn it_detects_loc_with_wrong_owner_as_invalid() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_identity_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));
        assert_eq!(LogionLoc::loc_valid_with_owner(&LOC_ID, &LOC_OWNER2), false);
    });
}

#[test]
fn it_creates_logion_identity_loc() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        let snapshot = BalancesSnapshot::take(LOC_OWNER1, LOC_OWNER1);
        assert_ok!(LogionLoc::create_logion_identity_loc(RuntimeOrigin::signed(LOC_OWNER1), LOGION_IDENTITY_LOC_ID));

        assert!(LogionLoc::loc(LOGION_IDENTITY_LOC_ID).is_some());
        assert!(LogionLoc::identity_loc_locs(LOGION_IDENTITY_LOC_ID).is_none());

        check_no_fees(snapshot);
    });
}

#[test]
fn it_creates_and_links_logion_locs_to_identity_loc() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        let snapshot = BalancesSnapshot::take(LOC_OWNER1, LOC_OWNER1);
        assert_ok!(LogionLoc::create_logion_identity_loc(RuntimeOrigin::signed(LOC_OWNER1), LOGION_IDENTITY_LOC_ID));
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOGION_IDENTITY_LOC_ID));

        assert_ok!(LogionLoc::create_logion_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOGION_IDENTITY_LOC_ID));
        assert_ok!(LogionLoc::create_logion_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), OTHER_LOC_ID, LOGION_IDENTITY_LOC_ID));

        assert!(LogionLoc::loc(LOC_ID).is_some());
        assert!(LogionLoc::loc(OTHER_LOC_ID).is_some());
        assert!(LogionLoc::identity_loc_locs(LOGION_IDENTITY_LOC_ID).is_some());
        assert!(LogionLoc::identity_loc_locs(LOGION_IDENTITY_LOC_ID).unwrap().len() == 2);
        assert_eq!(LogionLoc::identity_loc_locs(LOGION_IDENTITY_LOC_ID).unwrap()[0], LOC_ID);
        assert_eq!(LogionLoc::identity_loc_locs(LOGION_IDENTITY_LOC_ID).unwrap()[1], OTHER_LOC_ID);

        check_no_fees(snapshot);
    });
}

#[test]
fn it_fails_creating_logion_loc_with_polkadot_identity_loc() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_identity_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), OTHER_LOC_ID, LOC_OWNER1));

        assert_err!(LogionLoc::create_logion_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, OTHER_LOC_ID), Error::<Test>::UnexpectedRequester);
    });
}

#[test]
fn it_fails_creating_logion_loc_with_polkadot_transaction_loc() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_identity_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), OTHER_LOC_ID, LOC_OWNER1));

        assert_err!(LogionLoc::create_logion_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, OTHER_LOC_ID), Error::<Test>::UnexpectedRequester);
    });
}

#[test]
fn it_fails_creating_logion_loc_with_logion_transaction_loc() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_logion_identity_loc(RuntimeOrigin::signed(LOC_OWNER1), LOGION_IDENTITY_LOC_ID));
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOGION_IDENTITY_LOC_ID));
        assert_ok!(LogionLoc::create_logion_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), OTHER_LOC_ID, LOGION_IDENTITY_LOC_ID));

        assert_err!(LogionLoc::create_logion_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, OTHER_LOC_ID), Error::<Test>::UnexpectedRequester);
    });
}

#[test]
fn it_fails_creating_logion_loc_with_open_logion_identity_loc() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_logion_identity_loc(RuntimeOrigin::signed(LOC_OWNER1), LOGION_IDENTITY_LOC_ID));

        assert_err!(LogionLoc::create_logion_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOGION_IDENTITY_LOC_ID), Error::<Test>::UnexpectedRequester);
    });
}

#[test]
fn it_fails_creating_logion_loc_with_closed_void_logion_identity_loc() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_logion_identity_loc(RuntimeOrigin::signed(LOC_OWNER1), LOGION_IDENTITY_LOC_ID));
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOGION_IDENTITY_LOC_ID));
        assert_ok!(LogionLoc::make_void(RuntimeOrigin::signed(LOC_OWNER1), LOGION_IDENTITY_LOC_ID));

        assert_err!(LogionLoc::create_logion_transaction_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, LOGION_IDENTITY_LOC_ID), Error::<Test>::UnexpectedRequester);
    });
}

#[test]
fn it_creates_collection_loc() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        let snapshot = BalancesSnapshot::take(LOC_REQUESTER_ID, LOC_OWNER1);
        assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1, None, Some(10), false));
        assert_eq!(LogionLoc::loc(LOC_ID), Some(LegalOfficerCase {
            owner: LOC_OWNER1,
            requester: LOC_REQUESTER,
            metadata: vec![],
            files: vec![],
            closed: false,
            loc_type: LocType::Collection,
            links: vec![],
            void_info: None,
            replacer_of: None,
            collection_last_block_submission: None,
            collection_max_size: Some(10),
            collection_can_upload: false,
            seal: None,
            sponsorship_id: None,
        }));

        let fees = Fees::only_legal(2000 * ONE_LGNT, Beneficiary::LegalOfficer(LOC_OWNER1));
        fees.assert_balances_events(snapshot);
    });
}

#[test]
fn it_fails_creating_collection_loc_without_limit() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_err!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1, None, None, false), Error::<Test>::CollectionHasNoLimit);
    });
}

#[test]
fn it_fails_adding_item_to_open_collection_loc() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1, None, Some(10), false));
        let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
        let collection_item_description = sha256(&"item-description".as_bytes().to_vec());
        assert_err!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, collection_item_id, collection_item_description, vec![], None, false, Vec::new()), Error::<Test>::WrongCollectionLoc);
    });
}

#[test]
fn it_adds_item_to_closed_collection_loc() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1, None, Some(10), false));
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

        let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
        let collection_item_description = sha256(&"item-description".as_bytes().to_vec());
        assert_ok!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description.clone(), vec![], None, false, Vec::new()));
        assert_eq!(LogionLoc::collection_items(LOC_ID, collection_item_id), Some(CollectionItem {
            description: collection_item_description,
            files: vec![],
            token: None,
            restricted_delivery: false,
            terms_and_conditions: vec![],
        }));
        assert_eq!(LogionLoc::collection_size(LOC_ID), Some(1));
    });
}

#[test]
fn it_fails_to_item_with_terms_and_conditions_when_non_existent_tc_loc() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1, None, Some(10), false));
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

        let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
        let collection_item_description = sha256(&"item-description".as_bytes().to_vec());
        let terms_and_conditions_details = "ITEM-A, ITEM-B".as_bytes().to_vec();
        let terms_and_conditions = vec![TermsAndConditionsElement {
            tc_type: sha256(&"Logion".as_bytes().to_vec()),
            tc_loc: LOGION_CLASSIFICATION_LOC_ID,
            details: sha256(&terms_and_conditions_details),
        }];
        assert_err!(LogionLoc::add_collection_item_with_terms_and_conditions(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description.clone(), vec![], None, false, terms_and_conditions), Error::<Test>::TermsAndConditionsLocNotFound);
    });
}

#[test]
fn it_fails_to_item_with_terms_and_conditions_when_open_tc_loc() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1, None, Some(10), false));
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOGION_CLASSIFICATION_LOC_ID, LOC_OWNER1));

        let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
        let collection_item_description = sha256(&"item-description".as_bytes().to_vec());
        let terms_and_conditions_details = sha256(&"ITEM-A, ITEM-B".as_bytes().to_vec());
        let terms_and_conditions = vec![TermsAndConditionsElement {
            tc_type: sha256(&"Logion".as_bytes().to_vec()),
            tc_loc: LOGION_CLASSIFICATION_LOC_ID,
            details: terms_and_conditions_details.clone(),
        }];
        assert_err!(LogionLoc::add_collection_item_with_terms_and_conditions(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description.clone(), vec![], None, false, terms_and_conditions), Error::<Test>::TermsAndConditionsLocNotClosed);
    });
}

#[test]
fn it_fails_to_item_with_terms_and_conditions_when_void_tc_loc() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1, None, Some(10), false));
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOGION_CLASSIFICATION_LOC_ID, LOC_OWNER1));
        assert_ok!(LogionLoc::make_void(RuntimeOrigin::signed(LOC_OWNER1), LOGION_CLASSIFICATION_LOC_ID));

        let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
        let collection_item_description = sha256(&"item-description".as_bytes().to_vec());
        let terms_and_conditions_details = sha256(&"ITEM-A, ITEM-B".as_bytes().to_vec());
        let terms_and_conditions = vec![TermsAndConditionsElement {
            tc_type: sha256(&"Logion".as_bytes().to_vec()),
            tc_loc: LOGION_CLASSIFICATION_LOC_ID,
            details: terms_and_conditions_details,
        }];
        assert_err!(LogionLoc::add_collection_item_with_terms_and_conditions(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description.clone(), vec![], None, false, terms_and_conditions), Error::<Test>::TermsAndConditionsLocVoid);
    });
}

#[test]
fn it_adds_item_with_terms_and_conditions_to_closed_collection_loc() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1, None, Some(10), false));
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOGION_CLASSIFICATION_LOC_ID, LOC_OWNER1));
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOGION_CLASSIFICATION_LOC_ID));
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), ADDITIONAL_TC_LOC_ID, LOC_OWNER1));
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), ADDITIONAL_TC_LOC_ID));

        let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
        let collection_item_description = sha256(&"item-description".as_bytes().to_vec());
        let tc1 = TermsAndConditionsElement {
            tc_type: sha256(&"Logion".as_bytes().to_vec()),
            tc_loc: LOGION_CLASSIFICATION_LOC_ID,
            details: sha256(&"ITEM-A, ITEM-B".as_bytes().to_vec()),
        };
        let tc2 = TermsAndConditionsElement {
            tc_type: sha256(&"Specific".as_bytes().to_vec()),
            tc_loc: ADDITIONAL_TC_LOC_ID,
            details: sha256(&"Some more details".as_bytes().to_vec()),
        };
        let terms_and_conditions = vec![tc1, tc2];
        assert_ok!(LogionLoc::add_collection_item_with_terms_and_conditions(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description.clone(), vec![], None, false, terms_and_conditions.clone()));
        assert_eq!(LogionLoc::collection_items(LOC_ID, collection_item_id), Some(CollectionItem {
            description: collection_item_description,
            files: vec![],
            token: None,
            restricted_delivery: false,
            terms_and_conditions: terms_and_conditions.clone(),
        }));
        assert_eq!(LogionLoc::collection_size(LOC_ID), Some(1));
    });
}

#[test]
fn it_fails_adding_item_to_collection_loc_if_not_requester() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1, None, Some(10), false));
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

        let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
        let collection_item_description = sha256(&"item-description".as_bytes().to_vec());
        assert_err!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, collection_item_id, collection_item_description, vec![], None, false, Vec::new()), Error::<Test>::WrongCollectionLoc);
    });
}

#[test]
fn it_fails_adding_item_if_duplicate_key() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1, None, Some(10), false));
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

        let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
        let collection_item_description = sha256(&"item-description".as_bytes().to_vec());
        assert_ok!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id.clone(), collection_item_description.clone(), vec![], None, false, Vec::new()));
        assert_err!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description, vec![], None, false, Vec::new()), Error::<Test>::CollectionItemAlreadyExists);
    });
}

#[test]
fn it_fails_adding_item_if_size_limit_reached() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1, None, Some(1), false));
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

        let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
        let collection_item_description = sha256(&"item-description".as_bytes().to_vec());
        assert_ok!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id.clone(), collection_item_description.clone(), vec![], None, false, Vec::new()));
        let collection_item_id2 = BlakeTwo256::hash_of(&"item-id2".as_bytes().to_vec());
        assert_err!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id2, collection_item_description, vec![], None, false, Vec::new()), Error::<Test>::CollectionLimitsReached);
    });
}

#[test]
fn it_fails_adding_item_if_block_limit_reached() {
    let current_block: u64 = 10;
    new_test_ext_at_block(current_block).execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1, Some(current_block - 1), None, false));
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

        let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
        let collection_item_description = sha256(&"item-description".as_bytes().to_vec());
        assert_err!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description, vec![], None, false, Vec::new()), Error::<Test>::CollectionLimitsReached);
    });
}

#[test]
fn it_fails_adding_item_if_collection_void() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1, None, Some(1), false));
        assert_ok!(LogionLoc::make_void(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

        let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
        let collection_item_description = sha256(&"item-description".as_bytes().to_vec());
        assert_err!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description, vec![], None, false, Vec::new()), Error::<Test>::WrongCollectionLoc);
    });
}

#[test]
fn it_fails_adding_item_if_files_attached_but_upload_not_enabled() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1, None, Some(1), false));
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

        let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
        let collection_item_description = sha256(&"item-description".as_bytes().to_vec());
        let collection_item_files = vec![CollectionItemFile {
            name: sha256(&"picture.png".as_bytes().to_vec()),
            content_type: sha256(&"image/png".as_bytes().to_vec()),
            hash: BlakeTwo256::hash_of(&"file content".as_bytes().to_vec()),
            size: 123456,
        }];
        assert_err!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description, collection_item_files, None, false, Vec::new()), Error::<Test>::CannotUpload);
    });
}

#[test]
fn it_adds_item_if_no_files_attached_and_upload_enabled() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1, None, Some(1), true));
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

        let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
        let collection_item_description = sha256(&"item-description".as_bytes().to_vec());
        assert_ok!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description, vec![], None, false, Vec::new()));
    });
}

#[test]
fn it_adds_item_with_one_file_attached() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1, None, Some(1), true));
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

        let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
        let collection_item_description = sha256(&"item-description".as_bytes().to_vec());
        let collection_item_files = vec![CollectionItemFile {
            name: sha256(&"picture.png".as_bytes().to_vec()),
            content_type: sha256(&"image/png".as_bytes().to_vec()),
            hash: BlakeTwo256::hash_of(&"file content".as_bytes().to_vec()),
            size: FILE_SIZE,
        }];
        let snapshot = BalancesSnapshot::take(LOC_REQUESTER_ID, LOC_OWNER1);
        assert_ok!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description, collection_item_files, None, false, Vec::new()));
        let fees = Fees::only_storage(1, FILE_SIZE);
        fees.assert_balances_events(snapshot);
    });
}

#[test]
fn it_fails_adding_item_with_insufficient_balance() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1, None, Some(1), true));
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));
        set_balance(LOC_REQUESTER_ID, INSUFFICIENT_BALANCE);

        let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
        let collection_item_description = sha256(&"item-description".as_bytes().to_vec());
        let collection_item_files = vec![CollectionItemFile {
            name: sha256(&"picture.png".as_bytes().to_vec()),
            content_type: sha256(&"image/png".as_bytes().to_vec()),
            hash: BlakeTwo256::hash_of(&"file content".as_bytes().to_vec()),
            size: FILE_SIZE,
        }];
        let snapshot = BalancesSnapshot::take(LOC_REQUESTER_ID, LOC_OWNER1);
        assert_err!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description, collection_item_files, None, false, Vec::new()), Error::<Test>::InsufficientFunds);
        check_no_fees(snapshot);
    });
}

#[test]
fn it_adds_item_with_token() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1, None, Some(1), true));
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

        let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
        let collection_item_description = sha256(&"item-description".as_bytes().to_vec());
        let collection_item_files = vec![CollectionItemFile {
            name: sha256(&"picture.png".as_bytes().to_vec()),
            content_type: sha256(&"image/png".as_bytes().to_vec()),
            hash: BlakeTwo256::hash_of(&"file content".as_bytes().to_vec()),
            size: 123456,
        }];
        let collection_item_token = CollectionItemToken {
            token_type: sha256(&"ethereum_erc721".as_bytes().to_vec()),
            token_id: sha256(&"{\"contract\":\"0x765df6da33c1ec1f83be42db171d7ee334a46df5\",\"token\":\"4391\"}".as_bytes().to_vec()),
            token_issuance: 2,
        };
        let snapshot = BalancesSnapshot::take(LOC_REQUESTER_ID, LOC_OWNER1);
        assert_ok!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description, collection_item_files.clone(), Some(collection_item_token), true, Vec::new()));
        let fees = Fees {
            storage_fees: Fees::storage_fees(1, collection_item_files[0].size),
            legal_fees: 0,
            legal_fee_beneficiary: None,
            certificate_fees: 8_000_000_000_000_000,
        };
        fees.assert_balances_events(snapshot);
    });
}

#[test]
fn it_fails_adding_item_with_missing_token() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1, None, Some(1), true));
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

        let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
        let collection_item_description = sha256(&"item-description".as_bytes().to_vec());
        let collection_item_files = vec![CollectionItemFile {
            name: sha256(&"picture.png".as_bytes().to_vec()),
            content_type: sha256(&"image/png".as_bytes().to_vec()),
            hash: BlakeTwo256::hash_of(&"file content".as_bytes().to_vec()),
            size: 123456,
        }];
        assert_err!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description, collection_item_files, None, true, Vec::new()), Error::<Test>::MissingToken);
    });
}

#[test]
fn it_fails_adding_item_with_missing_files() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1, None, Some(1), true));
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

        let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
        let collection_item_description = sha256(&"item-description".as_bytes().to_vec());
        let collection_item_files = vec![];
        let collection_item_token = CollectionItemToken {
            token_type: sha256(&"ethereum_erc721".as_bytes().to_vec()),
            token_id: sha256(&"{\"contract\":\"0x765df6da33c1ec1f83be42db171d7ee334a46df5\",\"token\":\"4391\"}".as_bytes().to_vec()),
            token_issuance: 1,
        };
        assert_err!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description, collection_item_files, Some(collection_item_token), true, Vec::new()), Error::<Test>::MissingFiles);
    });
}

#[test]
fn it_adds_item_with_two_files_attached() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1, None, Some(1), true));
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

        let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
        let collection_item_description = sha256(&"item-description".as_bytes().to_vec());
        let collection_item_files = vec![
            CollectionItemFile {
                name: sha256(&"picture.png".as_bytes().to_vec()),
                content_type: sha256(&"image/png".as_bytes().to_vec()),
                hash: BlakeTwo256::hash_of(&"file content".as_bytes().to_vec()),
                size: 123456,
            },
            CollectionItemFile {
                name: sha256(&"doc.pdf".as_bytes().to_vec()),
                content_type: sha256(&"application/pdf".as_bytes().to_vec()),
                hash: BlakeTwo256::hash_of(&"some other content".as_bytes().to_vec()),
                size: 789,
            },
        ];
        let snapshot = BalancesSnapshot::take(LOC_REQUESTER_ID, LOC_OWNER1);
        assert_ok!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description, collection_item_files, None, false, Vec::new()));

        let fees = Fees::only_storage(2, 123456 + 789);
        fees.assert_balances_events(snapshot);
    });
}

#[test]
fn it_fails_to_add_item_with_duplicate_hash() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1, None, Some(1), true));
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

        let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
        let collection_item_description = sha256(&"item-description".as_bytes().to_vec());
        let same_hash = BlakeTwo256::hash_of(&"file content".as_bytes().to_vec());
        let collection_item_files = vec![
            CollectionItemFile {
                name: sha256(&"picture.png".as_bytes().to_vec()),
                content_type: sha256(&"image/png".as_bytes().to_vec()),
                hash: same_hash,
                size: 123456,
            },
            CollectionItemFile {
                name: sha256(&"doc.pdf".as_bytes().to_vec()),
                content_type: sha256(&"application/pdf".as_bytes().to_vec()),
                hash: same_hash,
                size: 789,
            },
        ];
        assert_err!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description, collection_item_files, None, false, Vec::new()), Error::<Test>::DuplicateFile);
    });
}

#[test]
fn it_closes_and_seals_loc() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        let seal = BlakeTwo256::hash_of(&"some external private data".as_bytes().to_vec());
        assert_ok!(LogionLoc::close_and_seal(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, seal));
        let loc = LogionLoc::loc(LOC_ID).unwrap();
        assert!(loc.closed);
        assert!(loc.seal.is_some());
        assert_eq!(loc.seal.unwrap(), seal);
    });
}

#[test]
fn it_fails_adding_file_with_same_hash() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        let file1 = FileParams {
            hash: sha256(&"test".as_bytes().to_vec()),
            nature: sha256(&"test-file-nature".as_bytes().to_vec()),
            submitter: SupportedAccountId::Polkadot(LOC_REQUESTER_ID),
            size: FILE_SIZE,
        };
        let snapshot = BalancesSnapshot::take(LOC_REQUESTER_ID, LOC_OWNER1);
        assert_ok!(LogionLoc::add_file(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, file1.clone()));
        let file2 = FileParams {
            hash: sha256(&"test".as_bytes().to_vec()),
            nature: sha256(&"test-file2-nature".as_bytes().to_vec()),
            submitter: SupportedAccountId::Polkadot(LOC_REQUESTER_ID),
            size: FILE_SIZE,
        };
        assert_err!(LogionLoc::add_file(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, file2.clone()), Error::<Test>::DuplicateLocFile);
        let fees = Fees::only_storage(1, FILE_SIZE);
        fees.assert_balances_events(snapshot);
    });
}

#[test]
fn it_fails_adding_metadata_with_same_name() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        let metadata1 = MetadataItemParams {
            name: sha256(&"name".as_bytes().to_vec()),
            value: sha256(&"value1".as_bytes().to_vec()),
            submitter: SupportedAccountId::Polkadot(LOC_REQUESTER_ID),
        };
        assert_ok!(LogionLoc::add_metadata(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, metadata1.clone()));
        let metadata2 = MetadataItemParams {
            name: sha256(&"name".as_bytes().to_vec()),
            value: sha256(&"value2".as_bytes().to_vec()),
            submitter: SupportedAccountId::Polkadot(LOC_REQUESTER_ID),
        };
        assert_err!(LogionLoc::add_metadata(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, metadata2.clone()), Error::<Test>::DuplicateLocMetadata);
    });
}

#[test]
fn it_fails_adding_link_with_same_target() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), OTHER_LOC_ID, LOC_OWNER1));
        let link1 = LocLink {
            id: OTHER_LOC_ID,
            nature: sha256(&"test-link1-nature".as_bytes().to_vec()),
        };
        assert_ok!(LogionLoc::add_link(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, link1.clone()));
        let link2 = LocLink {
            id: OTHER_LOC_ID,
            nature: sha256(&"test-link2-nature".as_bytes().to_vec()),
        };
        assert_err!(LogionLoc::add_link(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, link2.clone()), Error::<Test>::DuplicateLocLink);
    });
}

#[test]
fn it_adds_several_metadata() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        let metadata1 = MetadataItemParams {
            name: sha256(&vec![1, 2, 3]),
            value: sha256(&vec![4, 5, 6]),
            submitter: SupportedAccountId::Polkadot(LOC_OWNER1),
        };
        assert_ok!(LogionLoc::add_metadata(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, metadata1.clone()));
        let metadata2 = MetadataItemParams {
            name: sha256(&vec![1, 2, 4]),
            value: sha256(&vec![4, 5, 6]),
            submitter: SupportedAccountId::Polkadot(LOC_OWNER1),
        };
        assert_ok!(LogionLoc::add_metadata(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, metadata2.clone()));
        let loc = LogionLoc::loc(LOC_ID).unwrap();
        assert_eq!(loc.metadata[0], expected_metadata(metadata1, ACKNOWLEDGED));
        assert_eq!(loc.metadata[1], expected_metadata(metadata2, ACKNOWLEDGED));
    });
}

#[test]
fn it_nominates_an_issuer() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        nominate_issuer(ISSUER_ID1, ISSUER1_IDENTITY_LOC_ID);

        assert_eq!(LogionLoc::verified_issuers(LOC_OWNER1, ISSUER_ID1), Some(VerifiedIssuer { identity_loc: ISSUER1_IDENTITY_LOC_ID }));
    });
}

#[test]
fn it_dismisses_an_issuer() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        nominate_issuer(ISSUER_ID1, ISSUER1_IDENTITY_LOC_ID);

        assert_ok!(LogionLoc::dismiss_issuer(RuntimeOrigin::signed(LOC_OWNER1), ISSUER_ID1));

        assert_eq!(LogionLoc::verified_issuers(LOC_OWNER1, ISSUER_ID1), None);
    });
}

fn nominate_issuer(issuer: u64, identity_loc: u32) {
    assert_ok!(LogionLoc::create_polkadot_identity_loc(RuntimeOrigin::signed(issuer), identity_loc, LOC_OWNER1));
    assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), identity_loc));
    assert_ok!(LogionLoc::nominate_issuer(RuntimeOrigin::signed(LOC_OWNER1), issuer, identity_loc));
}

#[test]
fn it_selects_an_issuer() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        create_collection_and_nominated_issuer();

        assert_ok!(LogionLoc::set_issuer_selection(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, ISSUER_ID1, true));

        assert_eq!(LogionLoc::verified_issuers_by_loc(LOC_ID, ISSUER_ID1), Some(()));
        assert_eq!(LogionLoc::locs_by_verified_issuer((ISSUER_ID1, LOC_OWNER1, LOC_ID)), Some(()));
    });
}

fn create_collection_and_nominated_issuer() {
    assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1, None, Some(10), true));
    nominate_issuer(ISSUER_ID1, ISSUER1_IDENTITY_LOC_ID);
}

#[test]
fn it_fails_selecting_an_issuer_loc_not_found() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_err!(LogionLoc::set_issuer_selection(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, ISSUER_ID1, true), Error::<Test>::NotFound);
    });
}

#[test]
fn it_fails_selecting_an_issuer_not_nominated() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1, None, Some(10), true));

        assert_err!(LogionLoc::set_issuer_selection(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, ISSUER_ID1, true), Error::<Test>::NotNominated);
    });
}

#[test]
fn it_fails_selecting_an_issuer_unauthorized() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        create_collection_and_nominated_issuer();

        assert_err!(LogionLoc::set_issuer_selection(RuntimeOrigin::signed(LOC_OWNER2), LOC_ID, ISSUER_ID1, true), Error::<Test>::Unauthorized);
    });
}

#[test]
fn it_selects_an_issuer_closed() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        create_collection_and_nominated_issuer();
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

        assert_ok!(LogionLoc::set_issuer_selection(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, ISSUER_ID1, true));
    });
}

#[test]
fn it_fails_selecting_an_issuer_void() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        create_collection_and_nominated_issuer();
        assert_ok!(LogionLoc::make_void(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

        assert_err!(LogionLoc::set_issuer_selection(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, ISSUER_ID1, true), Error::<Test>::CannotMutateVoid);
    });
}

#[test]
fn it_selects_an_issuer_not_collection() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        nominate_issuer(ISSUER_ID1, ISSUER1_IDENTITY_LOC_ID);
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));

        assert_ok!(LogionLoc::set_issuer_selection(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, ISSUER_ID1, true));
    });
}

#[test]
fn it_unselects_an_issuer() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        create_collection_with_selected_issuer();

        assert_ok!(LogionLoc::set_issuer_selection(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, ISSUER_ID1, false));

        assert_eq!(LogionLoc::verified_issuers_by_loc(LOC_ID, ISSUER_ID1), None);
        assert_eq!(LogionLoc::locs_by_verified_issuer((ISSUER_ID1, LOC_OWNER1, LOC_ID)), None);
    });
}

fn create_collection_with_selected_issuer() {
    create_collection_and_nominated_issuer();
    assert_ok!(LogionLoc::set_issuer_selection(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, ISSUER_ID1, true));
}

#[test]
fn it_fails_unselecting_an_issuer_loc_not_found() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_err!(LogionLoc::set_issuer_selection(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, ISSUER_ID1, false), Error::<Test>::NotFound);
    });
}

#[test]
fn it_fails_unselecting_an_issuer_unauthorized() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        create_collection_with_selected_issuer();

        assert_err!(LogionLoc::set_issuer_selection(RuntimeOrigin::signed(LOC_OWNER2), LOC_ID, ISSUER_ID1, false), Error::<Test>::Unauthorized);
    });
}

#[test]
fn it_unselects_an_issuer_closed() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        create_collection_with_selected_issuer();
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

        assert_ok!(LogionLoc::set_issuer_selection(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, ISSUER_ID1, false));
    });
}

#[test]
fn it_fails_unselecting_an_issuer_void() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        create_collection_with_selected_issuer();
        assert_ok!(LogionLoc::make_void(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

        assert_err!(LogionLoc::set_issuer_selection(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, ISSUER_ID1, false), Error::<Test>::CannotMutateVoid);
    });
}

#[test]
fn it_unselects_issuer_on_dismiss() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        create_closed_collection_with_selected_issuer();
        nominate_issuer(ISSUER_ID2, ISSUER2_IDENTITY_LOC_ID);
        assert_ok!(LogionLoc::set_issuer_selection(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, ISSUER_ID2, true));
        assert!(LogionLoc::verified_issuers_by_loc(LOC_ID, ISSUER_ID1).is_some());
        assert!(LogionLoc::verified_issuers_by_loc(LOC_ID, ISSUER_ID2).is_some());
        assert!(LogionLoc::locs_by_verified_issuer((ISSUER_ID1, LOC_OWNER1, LOC_ID)).is_some());
        assert!(LogionLoc::locs_by_verified_issuer((ISSUER_ID2, LOC_OWNER1, LOC_ID)).is_some());

        assert_ok!(LogionLoc::dismiss_issuer(RuntimeOrigin::signed(LOC_OWNER1), ISSUER_ID1));

        assert!(LogionLoc::verified_issuers_by_loc(LOC_ID, ISSUER_ID1).is_none());
        assert!(LogionLoc::verified_issuers_by_loc(LOC_ID, ISSUER_ID2).is_some());
        assert!(LogionLoc::locs_by_verified_issuer((ISSUER_ID1, LOC_OWNER1, LOC_ID)).is_none());
        assert!(LogionLoc::locs_by_verified_issuer((ISSUER_ID2, LOC_OWNER1, LOC_ID)).is_some());
    });
}

#[test]
fn it_adds_tokens_record_issuer() {
    it_adds_tokens_record(ISSUER_ID1);
}

fn it_adds_tokens_record(submitter: AccountId) {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        create_closed_collection_with_selected_issuer();
        let record_id = build_record_id();
        let record_description = build_record_description();
        let record_files = build_record_files(1);

        let snapshot = BalancesSnapshot::take(LOC_REQUESTER_ID, LOC_OWNER1);
        assert_ok!(LogionLoc::add_tokens_record(RuntimeOrigin::signed(submitter), LOC_ID, record_id, record_description.clone(), record_files.clone()));

        let record = LogionLoc::tokens_records(LOC_ID, record_id).unwrap();
        assert_eq!(record.description, record_description);
        assert_eq!(record.submitter, submitter);
        assert_eq!(record.files.len(), 1);
        assert_eq!(record.files[0].name, record_files[0].name);
        assert_eq!(record.files[0].content_type, record_files[0].content_type);
        assert_eq!(record.files[0].size, record_files[0].size);
        assert_eq!(record.files[0].hash, record_files[0].hash);

        let fees = Fees::only_storage(1, record_files[0].size);
        fees.assert_balances_events(snapshot);
    });
}

fn create_closed_collection_with_selected_issuer() {
    create_collection_with_selected_issuer();
    assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));
}

fn build_record_id() -> H256 {
    BlakeTwo256::hash_of(&"Record ID".as_bytes().to_vec())
}

fn build_record_description() -> H256 {
    sha256(&"Some description".as_bytes().to_vec())
}

fn build_record_files(files: usize) -> Vec<TokensRecordFileOf<Test>> {
    let mut record_files = Vec::with_capacity(files);
    for i in 0..files {
        let file = TokensRecordFile {
            name: sha256(&"File name".as_bytes().to_vec()),
            content_type: sha256(&"text/plain".as_bytes().to_vec()),
            size: i as u32 % 10,
            hash: BlakeTwo256::hash_of(&i.to_string().as_bytes().to_vec()),
        };
        record_files.push(file);
    }
    record_files
}

#[test]
fn it_adds_tokens_record_requester() {
    it_adds_tokens_record(LOC_REQUESTER_ID);
}

#[test]
fn it_adds_tokens_record_owner() {
    it_adds_tokens_record(LOC_OWNER1);
}

#[test]
fn it_fails_adding_tokens_record_already_exists() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        create_closed_collection_with_selected_issuer();
        let record_id = build_record_id();
        let record_description = build_record_description();
        let record_files = build_record_files(1);

        let snapshot = BalancesSnapshot::take(LOC_REQUESTER_ID, LOC_OWNER1);
        assert_ok!(LogionLoc::add_tokens_record(RuntimeOrigin::signed(ISSUER_ID1), LOC_ID, record_id, record_description.clone(), record_files.clone()));
        assert_err!(LogionLoc::add_tokens_record(RuntimeOrigin::signed(ISSUER_ID1), LOC_ID, record_id, record_description, record_files.clone()), Error::<Test>::TokensRecordAlreadyExists);
        let file = record_files.get(0).unwrap();

        let fees = Fees::only_storage(1, file.size);
        fees.assert_balances_events(snapshot);
    });
}

#[test]
fn it_fails_adding_tokens_record_not_contributor() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        create_closed_collection_with_selected_issuer();
        let record_id = build_record_id();
        let record_description = build_record_description();
        let record_files = build_record_files(1);

        assert_err!(LogionLoc::add_tokens_record(RuntimeOrigin::signed(ISSUER_ID2), LOC_ID, record_id, record_description, record_files), Error::<Test>::CannotAddRecord);
    });
}

#[test]
fn it_fails_adding_tokens_record_collection_open() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        create_collection_with_selected_issuer();
        let record_id = build_record_id();
        let record_description = build_record_description();
        let record_files = build_record_files(1);

        assert_err!(LogionLoc::add_tokens_record(RuntimeOrigin::signed(ISSUER_ID1), LOC_ID, record_id, record_description, record_files), Error::<Test>::CannotAddRecord);
    });
}

#[test]
fn it_fails_adding_tokens_record_collection_void() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        create_collection_with_selected_issuer();
        assert_ok!(LogionLoc::make_void(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));
        let record_id = build_record_id();
        let record_description = build_record_description();
        let record_files = build_record_files(1);

        assert_err!(LogionLoc::add_tokens_record(RuntimeOrigin::signed(ISSUER_ID1), LOC_ID, record_id, record_description, record_files), Error::<Test>::CannotAddRecord);
    });
}

#[test]
fn it_fails_adding_tokens_record_not_collection() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        let record_id = build_record_id();
        let record_description = build_record_description();
        let record_files = build_record_files(1);

        assert_err!(LogionLoc::add_tokens_record(RuntimeOrigin::signed(ISSUER_ID1), LOC_ID, record_id, record_description, record_files), Error::<Test>::CannotAddRecord);
    });
}

#[test]
fn it_fails_adding_tokens_record_no_files() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        create_closed_collection_with_selected_issuer();
        let record_id = build_record_id();
        let record_description = build_record_description();
        let record_files = vec![];

        assert_err!(LogionLoc::add_tokens_record(RuntimeOrigin::signed(ISSUER_ID1), LOC_ID, record_id, record_description, record_files), Error::<Test>::MustUpload);
    });
}

#[test]
fn it_fails_adding_tokens_record_duplicate_file() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        create_closed_collection_with_selected_issuer();
        let record_id = build_record_id();
        let record_description = build_record_description();
        let file1 = TokensRecordFile {
            name: sha256(&"File name".as_bytes().to_vec()),
            content_type: sha256(&"text/plain".as_bytes().to_vec()),
            size: 4,
            hash: BlakeTwo256::hash_of(&"test".as_bytes().to_vec()),
        };
        let file2 = TokensRecordFile {
            name: sha256(&"File name 2".as_bytes().to_vec()),
            content_type: sha256(&"text/plain".as_bytes().to_vec()),
            size: 4,
            hash: BlakeTwo256::hash_of(&"test".as_bytes().to_vec()),
        };
        let record_files = vec![file1, file2];

        assert_err!(LogionLoc::add_tokens_record(RuntimeOrigin::signed(ISSUER_ID1), LOC_ID, record_id, record_description, record_files), Error::<Test>::DuplicateFile);
    });
}

#[test]
fn it_fails_adding_tokens_record_too_many_files() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        create_closed_collection_with_selected_issuer();
        let record_id = build_record_id();
        let record_description = build_record_description();
        let record_files = build_record_files(256);

        assert_err!(LogionLoc::add_tokens_record(RuntimeOrigin::signed(ISSUER_ID1), LOC_ID, record_id, record_description, record_files), Error::<Test>::TokensRecordTooMuchData);
    });
}

#[test]
fn it_fails_adding_tokens_record_when_insufficient_funds() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        create_closed_collection_with_selected_issuer();
        set_balance(LOC_REQUESTER_ID, INSUFFICIENT_BALANCE);
        let record_id = build_record_id();
        let record_description = build_record_description();
        let record_files = build_record_files(1);

        let snapshot = BalancesSnapshot::take(LOC_REQUESTER_ID, LOC_OWNER1);
        assert_err!(LogionLoc::add_tokens_record(RuntimeOrigin::signed(ISSUER_ID1), LOC_ID, record_id, record_description, record_files), Error::<Test>::InsufficientFunds);
        check_no_fees(snapshot);
    });
}

#[test]
fn it_adds_file_on_logion_identity_loc_when_caller_is_owner_and_submitter_is_issuer() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_logion_identity_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));
        nominated_and_select_issuer(LOC_ID);
        let file = FileParams {
            hash: sha256(&"test".as_bytes().to_vec()),
            nature: sha256(&"test-file-nature".as_bytes().to_vec()),
            submitter: SupportedAccountId::Polkadot(ISSUER_ID1),
            size: FILE_SIZE,
        };
        let snapshot = BalancesSnapshot::take(LOC_OWNER1, LOC_OWNER1);
        assert_ok!(LogionLoc::add_file(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, file.clone()));
        let loc = LogionLoc::loc(LOC_ID).unwrap();
        assert_eq!(loc.files[0], expected_file(&file, ACKNOWLEDGED));

        let fees = Fees::only_storage(1, FILE_SIZE);
        fees.assert_balances_events(snapshot);
    });
}

#[test]
fn it_adds_file_on_logion_identity_loc_when_caller_and_submitter_is_issuer() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_logion_identity_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));
        nominated_and_select_issuer(LOC_ID);
        let file = FileParams {
            hash: sha256(&"test".as_bytes().to_vec()),
            nature: sha256(&"test-file-nature".as_bytes().to_vec()),
            submitter: SupportedAccountId::Polkadot(ISSUER_ID1),
            size: FILE_SIZE,
        };
        let snapshot = BalancesSnapshot::take(LOC_OWNER1, LOC_OWNER1);
        assert_ok!(LogionLoc::add_file(RuntimeOrigin::signed(ISSUER_ID1), LOC_ID, file.clone()));
        let loc = LogionLoc::loc(LOC_ID).unwrap();
        assert_eq!(loc.files[0], expected_file(&file, NOT_ACKNOWLEDGED));

        let fees = Fees::only_storage(1, FILE_SIZE);
        fees.assert_balances_events(snapshot);
    });
}

fn nominated_and_select_issuer(loc_id: u32) {
    nominate_issuer(ISSUER_ID1, ISSUER1_IDENTITY_LOC_ID);
    assert_ok!(LogionLoc::set_issuer_selection(RuntimeOrigin::signed(LOC_OWNER1), loc_id, ISSUER_ID1, true));
}

#[test]
fn it_adds_file_on_polkadot_transaction_loc_when_caller_is_owner_and_submitter_is_issuer() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        nominated_and_select_issuer(LOC_ID);
        let file = FileParams {
            hash: sha256(&"test".as_bytes().to_vec()),
            nature: sha256(&"test-file-nature".as_bytes().to_vec()),
            submitter: SupportedAccountId::Polkadot(ISSUER_ID1),
            size: FILE_SIZE,
        };
        let snapshot = BalancesSnapshot::take(LOC_REQUESTER_ID, LOC_OWNER1);
        assert_ok!(LogionLoc::add_file(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, file.clone()));
        let loc = LogionLoc::loc(LOC_ID).unwrap();
        assert_eq!(loc.files[0], expected_file(&file, ACKNOWLEDGED));

        let fees = Fees::only_storage(1, FILE_SIZE);
        fees.assert_balances_events(snapshot);
    });
}

#[test]
fn it_adds_file_on_polkadot_transaction_loc_when_caller_is_submitter_and_issuer() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        nominated_and_select_issuer(LOC_ID);
        let file = FileParams {
            hash: sha256(&"test".as_bytes().to_vec()),
            nature: sha256(&"test-file-nature".as_bytes().to_vec()),
            submitter: SupportedAccountId::Polkadot(ISSUER_ID1),
            size: FILE_SIZE,
        };
        let snapshot = BalancesSnapshot::take(LOC_REQUESTER_ID, LOC_OWNER1);
        assert_ok!(LogionLoc::add_file(RuntimeOrigin::signed(ISSUER_ID1), LOC_ID, file.clone()));
        let loc = LogionLoc::loc(LOC_ID).unwrap();
        assert_eq!(loc.files[0], expected_file(&file, NOT_ACKNOWLEDGED));

        let fees = Fees::only_storage(1, FILE_SIZE);
        fees.assert_balances_events(snapshot);
    });
}

#[test]
fn it_fails_adding_file_on_polkadot_transaction_loc_cannot_submit() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        let file = FileParams {
            hash: sha256(&"test".as_bytes().to_vec()),
            nature: sha256(&"test-file-nature".as_bytes().to_vec()),
            submitter: SupportedAccountId::Polkadot(LOC_OWNER2),
            size: FILE_SIZE,
        };
        assert_err!(LogionLoc::add_file(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, file.clone()), Error::<Test>::CannotSubmit);
    });
}

#[test]
fn it_adds_metadata_on_logion_identity_loc_for_when_submitter_is_issuer() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_logion_identity_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));
        nominated_and_select_issuer(LOC_ID);
        let metadata = MetadataItemParams {
            name: sha256(&vec![1, 2, 3]),
            value: sha256(&vec![4, 5, 6]),
            submitter: SupportedAccountId::Polkadot(ISSUER_ID1),
        };
        assert_ok!(LogionLoc::add_metadata(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, metadata.clone()));
    });
}

#[test]
fn it_fails_adding_metadata_on_logion_identity_loc_cannot_submit() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_logion_identity_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));
        let metadata = MetadataItemParams {
            name: sha256(&vec![1, 2, 3]),
            value: sha256(&vec![4, 5, 6]),
            submitter: SupportedAccountId::Polkadot(LOC_OWNER2),
        };
        assert_err!(LogionLoc::add_metadata(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, metadata.clone()), Error::<Test>::CannotSubmit);
    });
}

#[test]
fn it_adds_metadata_on_polkadot_transaction_loc_when_submitter_is_issuer() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        nominated_and_select_issuer(LOC_ID);
        let metadata = MetadataItemParams {
            name: sha256(&vec![1, 2, 3]),
            value: sha256(&vec![4, 5, 6]),
            submitter: SupportedAccountId::Polkadot(ISSUER_ID1),
        };
        assert_ok!(LogionLoc::add_metadata(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, metadata.clone()));
    });
}

#[test]
fn it_fails_adding_metadata_on_polkadot_transaction_loc_cannot_submit() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_polkadot_transaction_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        let metadata = MetadataItemParams {
            name: sha256(&vec![1, 2, 3]),
            value: sha256(&vec![4, 5, 6]),
            submitter: SupportedAccountId::Polkadot(LOC_OWNER2),
        };
        assert_err!(LogionLoc::add_metadata(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, metadata.clone()), Error::<Test>::CannotSubmit);
    });
}

#[test]
fn it_creates_sponsorship() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        let sponsorship_id = 1;
        let beneficiary = H160::from_str("0x900edc98db53508e6742723988b872dd08cd09c2").unwrap();
        let sponsored_account = SupportedAccountId::Other(OtherAccountId::Ethereum(beneficiary));

        assert_ok!(LogionLoc::sponsor(RuntimeOrigin::signed(SPONSOR_ID), sponsorship_id, sponsored_account, LOC_OWNER1));

        let sponsorship = LogionLoc::sponsorship(sponsorship_id).unwrap();
        assert_eq!(sponsorship.legal_officer, LOC_OWNER1);
        assert_eq!(sponsorship.sponsor, SPONSOR_ID);
        assert_eq!(sponsorship.sponsored_account, sponsored_account);
        assert_eq!(sponsorship.loc_id, None);
    });
}

#[test]
fn it_fails_creating_sponsorship_with_duplicate_id() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        let sponsorship_id = 1;
        let beneficiary = H160::from_str("0x900edc98db53508e6742723988b872dd08cd09c2").unwrap();
        let sponsored_account = SupportedAccountId::Other(OtherAccountId::Ethereum(beneficiary));

        assert_ok!(LogionLoc::sponsor(RuntimeOrigin::signed(SPONSOR_ID), sponsorship_id, sponsored_account, LOC_OWNER1));
        assert_err!(LogionLoc::sponsor(RuntimeOrigin::signed(SPONSOR_ID), sponsorship_id, sponsored_account, LOC_OWNER1), Error::<Test>::AlreadyExists);
    });
}

#[test]
fn it_withdraws_unused_sponsorship() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        let sponsorship_id = 1;
        let beneficiary = H160::from_str("0x900edc98db53508e6742723988b872dd08cd09c2").unwrap();
        let sponsored_account = SupportedAccountId::Other(OtherAccountId::Ethereum(beneficiary));
        assert_ok!(LogionLoc::sponsor(RuntimeOrigin::signed(SPONSOR_ID), sponsorship_id, sponsored_account, LOC_OWNER1));
        assert!(LogionLoc::sponsorship(sponsorship_id).is_some());

        assert_ok!(LogionLoc::withdraw_sponsorship(RuntimeOrigin::signed(SPONSOR_ID), sponsorship_id));

        assert!(LogionLoc::sponsorship(sponsorship_id).is_none());
    });
}

#[test]
fn it_creates_ethereum_identity_loc() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        let ethereum_address = H160::from_str("0x590E9c11b1c2f20210b9b84dc2417B4A7955d4e6").unwrap();
        let requester_account_id = OtherAccountId::Ethereum(ethereum_address);
        let sponsorship_id = 1;
        let sponsored_account = SupportedAccountId::Other(OtherAccountId::Ethereum(ethereum_address));
        assert_ok!(LogionLoc::sponsor(RuntimeOrigin::signed(SPONSOR_ID), sponsorship_id, sponsored_account, LOC_OWNER1));
        let snapshot = BalancesSnapshot::take(SPONSOR_ID, LOC_OWNER1);
        assert_ok!(LogionLoc::create_other_identity_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, requester_account_id.clone(), sponsorship_id));
        assert_eq!(LogionLoc::loc(LOC_ID), Some(LegalOfficerCase {
            owner: LOC_OWNER1,
            requester: OtherAccount(requester_account_id.clone()),
            metadata: vec![],
            files: vec![],
            closed: false,
            loc_type: LocType::Identity,
            links: vec![],
            void_info: None,
            replacer_of: None,
            collection_last_block_submission: None,
            collection_max_size: None,
            collection_can_upload: false,
            seal: None,
            sponsorship_id: Some(sponsorship_id),
        }));
        assert_eq!(LogionLoc::other_account_locs(requester_account_id), Some(vec![LOC_ID]));
        assert_eq!(LogionLoc::sponsorship(sponsorship_id).unwrap().loc_id, Some(LOC_ID));
        System::assert_has_event(RuntimeEvent::LogionLoc(crate::Event::LocCreated { 0: LOC_ID }));

        let fees = Fees::only_legal(160 * ONE_LGNT, Beneficiary::Treasury);
        fees.assert_balances_events(snapshot);
    });
}

#[test]
fn it_creates_polkadot_identity_loc() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        let snapshot = BalancesSnapshot::take(LOC_REQUESTER_ID, LOC_OWNER1);
        assert_ok!(LogionLoc::create_polkadot_identity_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1));
        assert_eq!(LogionLoc::loc(LOC_ID), Some(LegalOfficerCase {
            owner: LOC_OWNER1,
            requester: Account(LOC_REQUESTER_ID),
            metadata: vec![],
            files: vec![],
            closed: false,
            loc_type: LocType::Identity,
            links: vec![],
            void_info: None,
            replacer_of: None,
            collection_last_block_submission: None,
            collection_max_size: None,
            collection_can_upload: false,
            seal: None,
            sponsorship_id: None,
        }));
        assert_eq!(LogionLoc::account_locs(LOC_REQUESTER_ID), Some(vec![LOC_ID]));
        System::assert_has_event(RuntimeEvent::LogionLoc(crate::Event::LocCreated { 0: LOC_ID }));

        let fees = Fees::only_legal(160 * ONE_LGNT, Beneficiary::Treasury);
        fees.assert_balances_events(snapshot);
    });
}

#[test]
fn it_fails_creating_ethereum_identity_loc_if_duplicate_loc_id() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        let ethereum_address = H160::from_str("0x590E9c11b1c2f20210b9b84dc2417B4A7955d4e6").unwrap();
        let requester_address = OtherAccountId::Ethereum(ethereum_address);
        let sponsorship_id = 1;
        let sponsored_account: SupportedAccountId<AccountId, H160> = SupportedAccountId::Other(requester_address);
        assert_ok!(LogionLoc::sponsor(RuntimeOrigin::signed(SPONSOR_ID), sponsorship_id, sponsored_account, LOC_OWNER1));
        assert_ok!(LogionLoc::create_other_identity_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, requester_address.clone(), sponsorship_id));
        assert_err!(LogionLoc::create_other_identity_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, requester_address.clone(), sponsorship_id), Error::<Test>::AlreadyExists);
    });
}

#[test]
fn it_fails_creating_several_ethereum_identity_loc_with_single_sponsorship() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        let ethereum_address = H160::from_str("0x590E9c11b1c2f20210b9b84dc2417B4A7955d4e6").unwrap();
        let requester_address = OtherAccountId::Ethereum(ethereum_address);
        let sponsorship_id = 1;
        let sponsored_account: SupportedAccountId<AccountId, H160> = SupportedAccountId::Other(requester_address);
        assert_ok!(LogionLoc::sponsor(RuntimeOrigin::signed(SPONSOR_ID), sponsorship_id, sponsored_account, LOC_OWNER1));
        assert_ok!(LogionLoc::create_other_identity_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, requester_address.clone(), sponsorship_id));
        assert_err!(LogionLoc::create_other_identity_loc(RuntimeOrigin::signed(LOC_OWNER1), OTHER_LOC_ID, requester_address.clone(), sponsorship_id), Error::<Test>::CannotLinkToSponsorship);
    });
}

#[test]
fn it_fails_withdrawing_used_sponsorship() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        let sponsorship_id = 1;
        let beneficiary = H160::from_str("0x900edc98db53508e6742723988b872dd08cd09c2").unwrap();
        let requester_address = OtherAccountId::Ethereum(beneficiary);
        let sponsored_account = SupportedAccountId::Other(OtherAccountId::Ethereum(beneficiary));
        assert_ok!(LogionLoc::sponsor(RuntimeOrigin::signed(SPONSOR_ID), sponsorship_id, sponsored_account, LOC_OWNER1));
        assert_ok!(LogionLoc::create_other_identity_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, requester_address, sponsorship_id));

        assert_err!(LogionLoc::withdraw_sponsorship(RuntimeOrigin::signed(SPONSOR_ID), sponsorship_id), Error::<Test>::AlreadyUsed);
    });
}

#[test]
fn it_adds_metadata_when_submitter_is_ethereum_requester() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        let ethereum_address = H160::from_str("0x590E9c11b1c2f20210b9b84dc2417B4A7955d4e6").unwrap();
        let requester_address = OtherAccountId::Ethereum(ethereum_address);
        let sponsorship_id = 1;
        let sponsored_account = SupportedAccountId::Other(requester_address);
        assert_ok!(LogionLoc::sponsor(RuntimeOrigin::signed(SPONSOR_ID), sponsorship_id, sponsored_account, LOC_OWNER1));
        assert_ok!(LogionLoc::create_other_identity_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, requester_address, sponsorship_id));
        let metadata = MetadataItemParams {
            name: sha256(&vec![1, 2, 3]),
            value: sha256(&vec![4, 5, 6]),
            submitter: sponsored_account,
        };
        assert_ok!(LogionLoc::add_metadata(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, metadata.clone()));
        let loc = LogionLoc::loc(LOC_ID).unwrap();
        assert_eq!(loc.metadata[0], expected_metadata(metadata, ACKNOWLEDGED));
    });
}

#[test]
fn it_adds_file_when_submitter_is_ethereum_requester() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        let requester = H160::from_str("0x900edc98db53508e6742723988b872dd08cd09c2").unwrap();
        let sponsorship_id = 1;
        let sponsored_account = SupportedAccountId::Other(OtherAccountId::Ethereum(requester));
        assert_ok!(LogionLoc::sponsor(RuntimeOrigin::signed(SPONSOR_ID), sponsorship_id, sponsored_account, LOC_OWNER1));
        assert_ok!(LogionLoc::create_other_identity_loc(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, OtherAccountId::Ethereum(requester), sponsorship_id));
        let file = FileParams {
            hash: sha256(&"test".as_bytes().to_vec()),
            nature: sha256(&"test-file-nature".as_bytes().to_vec()),
            submitter: SupportedAccountId::Other(OtherAccountId::Ethereum(requester)),
            size: FILE_SIZE,
        };
        let snapshot = BalancesSnapshot::take(SPONSOR_ID, LOC_OWNER1);
        assert_ok!(LogionLoc::add_file(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID, file.clone()));
        let loc = LogionLoc::loc(LOC_ID).unwrap();
        assert_eq!(loc.files[0], expected_file(&file, ACKNOWLEDGED));

        let fees = Fees::only_storage(1, file.size);
        fees.assert_balances_events(snapshot);
    });
}

#[test]
fn it_fails_adding_item_with_token_with_zero_issuance() {
    new_test_ext().execute_with(|| {
        setup_default_balances();
        assert_ok!(LogionLoc::create_collection_loc(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, LOC_OWNER1, None, Some(1), true));
        assert_ok!(LogionLoc::close(RuntimeOrigin::signed(LOC_OWNER1), LOC_ID));

        let collection_item_id = BlakeTwo256::hash_of(&"item-id".as_bytes().to_vec());
        let collection_item_description = sha256(&"item-description".as_bytes().to_vec());
        let collection_item_files = vec![CollectionItemFile {
            name: sha256(&"picture.png".as_bytes().to_vec()),
            content_type: sha256(&"image/png".as_bytes().to_vec()),
            hash: BlakeTwo256::hash_of(&"file content".as_bytes().to_vec()),
            size: 123456,
        }];
        let collection_item_token = CollectionItemToken {
            token_type: sha256(&"ethereum_erc721".as_bytes().to_vec()),
            token_id: sha256(&"{\"contract\":\"0x765df6da33c1ec1f83be42db171d7ee334a46df5\",\"token\":\"4391\"}".as_bytes().to_vec()),
            token_issuance: 0,
        };
        assert_err!(LogionLoc::add_collection_item(RuntimeOrigin::signed(LOC_REQUESTER_ID), LOC_ID, collection_item_id, collection_item_description, collection_item_files.clone(), Some(collection_item_token), true, Vec::new()), Error::<Test>::BadTokenIssuance);
    });
}
