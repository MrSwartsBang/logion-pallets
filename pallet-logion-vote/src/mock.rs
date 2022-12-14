use crate as pallet_logion_vote;
use frame_support::parameter_types;
use frame_support::traits::EnsureOrigin;
use sp_core::hash::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup}, testing::Header,
};
use frame_system::{self as system, Config};
use logion_shared::{IsLegalOfficer, LocValidity};

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

pub const LEGAL_OFFICER1: u64 = 1;
pub const LOC_ID: u32 = 1;

impl IsLegalOfficer<<Test as system::Config>::AccountId, RuntimeOrigin> for LoAuthorityListMock {
    fn is_legal_officer(account: &<Test as system::Config>::AccountId) -> bool {
        return *account == LEGAL_OFFICER1;
    }
}

pub struct LocValidityMock;

impl LocValidity<<Test as pallet_logion_vote::Config>::LocId, <Test as system::Config>::AccountId> for LocValidityMock {
    fn loc_valid_with_owner(loc_id: &<Test as pallet_logion_vote::Config>::LocId, legal_officer: &<Test as Config>::AccountId) -> bool {
        return *loc_id == LOC_ID && *legal_officer == LEGAL_OFFICER1;
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
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
