use frame_support::dispatch::DispatchResultWithPostInfo;
use crate as pallet_logion_vote;
use frame_support::parameter_types;
use frame_support::traits::EnsureOrigin;
use sp_core::hash::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup}, testing::Header,
};
use frame_system::{self as system, Config};
use logion_shared::{IsLegalOfficer, LegalOfficerCaseSummary, LegalOfficerCreation, LocQuery, LocValidity};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        LogionVote: pallet_logion_vote::{Pallet, Call, Storage, Event<T>},
    }
);

pub struct LoAuthorityListMock;

impl EnsureOrigin<RuntimeOrigin> for LoAuthorityListMock {
    type Success = <Test as system::Config>::AccountId;

    fn try_origin(o: <Test as system::Config>::RuntimeOrigin) -> Result<Self::Success, <Test as system::Config>::RuntimeOrigin> {
        <Self as IsLegalOfficer<<Test as system::Config>::AccountId, <Test as system::Config>::RuntimeOrigin>>::try_origin(o)
    }
}

pub const HOST_LEGAL_OFFICER: u64 = 1;
pub const LEGAL_OFFICER2: u64 = 2;
pub const APPLYING_GUEST_LEGAL_OFFICER: u64 = 3;
pub const LOC_ID: u32 = 1;

impl IsLegalOfficer<<Test as system::Config>::AccountId, RuntimeOrigin> for LoAuthorityListMock {

    fn legal_officers() -> Vec<<Test as Config>::AccountId> {
        vec![HOST_LEGAL_OFFICER, LEGAL_OFFICER2 ]
    }
}

pub struct LocValidityMock;

impl LocValidity<<Test as pallet_logion_vote::Config>::LocId, <Test as system::Config>::AccountId> for LocValidityMock {
    fn loc_valid_with_owner(loc_id: &<Test as pallet_logion_vote::Config>::LocId, legal_officer: &<Test as Config>::AccountId) -> bool {
        return *loc_id == LOC_ID && *legal_officer == HOST_LEGAL_OFFICER;
    }
}

pub struct LocQueryMock;

impl LocQuery<<Test as pallet_logion_vote::Config>::LocId, <Test as system::Config>::AccountId> for LocQueryMock {
    fn has_closed_identity_locs(_account: &<Test as Config>::AccountId, _legal_officer: &Vec<<Test as Config>::AccountId>) -> bool {
        false
    }

    fn get_loc(loc_id: &<Test as crate::Config>::LocId) -> Option<LegalOfficerCaseSummary<<Test as Config>::AccountId>> {
        if *loc_id == LOC_ID {
            return Some(LegalOfficerCaseSummary {
                owner: HOST_LEGAL_OFFICER,
                requester: Some(APPLYING_GUEST_LEGAL_OFFICER),
            })
        }
        return None
    }
}

pub struct LegalOfficerCreationMock;

impl LegalOfficerCreation<<Test as system::Config>::AccountId> for LegalOfficerCreationMock {
    fn add_guest_legal_officer(guest_legal_officer_id: <Test as Config>::AccountId, host_legal_officer_id: <Test as Config>::AccountId) -> DispatchResultWithPostInfo {
        if guest_legal_officer_id == APPLYING_GUEST_LEGAL_OFFICER && host_legal_officer_id == HOST_LEGAL_OFFICER {
            Ok(().into())
        } else {
            panic!()
        }
    }
}

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_logion_vote::Config for Test {
    type LocId = u32;
    type RuntimeEvent = RuntimeEvent;
    type IsLegalOfficer = LoAuthorityListMock;
    type LocValidity = LocValidityMock;
    type LocQuery = LocQueryMock;
    type LegalOfficerCreation = LegalOfficerCreationMock;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
