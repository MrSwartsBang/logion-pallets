use crate::{self as pallet_loc, RequesterOf};
use logion_shared::IsLegalOfficer;
use sp_core::hash::H256;
use frame_support::{parameter_types, traits::EnsureOrigin};
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup}, testing::Header,
};
use frame_system as system;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        LogionLoc: pallet_loc::{Pallet, Call, Storage, Event<T>},
    }
);

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

pub const LOC_OWNER1: u64 = 1;
pub const LOC_OWNER2: u64 = 2;
pub const LOC_REQUESTER_ID: u64 = 3;
pub const LOC_REQUESTER: RequesterOf<Test> = RequesterOf::<Test>::Account(LOC_REQUESTER_ID);
pub const LOGION_IDENTITY_LOC_ID: u32 = 4;
pub const ISSUER_ID1: u64 = 5;
pub const ISSUER_ID2: u64 = 6;

pub struct LoAuthorityListMock;
impl EnsureOrigin<RuntimeOrigin> for LoAuthorityListMock {
    type Success = <Test as system::Config>::AccountId;

    fn try_origin(o: <Test as system::Config>::RuntimeOrigin) -> Result<Self::Success, <Test as system::Config>::RuntimeOrigin> {
        <Self as IsLegalOfficer<<Test as system::Config>::AccountId, <Test as system::Config>::RuntimeOrigin>>::try_origin(o)
    }
}

impl IsLegalOfficer<<Test as system::Config>::AccountId, RuntimeOrigin> for LoAuthorityListMock {

    fn legal_officers() -> Vec<<Test as system::Config>::AccountId> {
        vec![ LOC_OWNER1, LOC_OWNER2 ]
    }
}

parameter_types! {
    pub const MaxMetadataItemNameSize: usize = 40;
    pub const MaxMetadataItemValueSize: usize = 4096;
    pub const MaxFileNatureSize: usize = 255;
    pub const MaxLinkNatureSize: usize = 255;
    pub const MaxCollectionItemDescriptionSize: usize = 4096;
    pub const MaxCollectionItemTokenIdSize: usize = 255;
    pub const MaxCollectionItemTokenTypeSize: usize = 255;
    pub const MaxTokensRecordDescriptionSize: u32 = 255;
    pub const MaxFileNameSize: u32 = 255;
    pub const MaxFileContentTypeSize: u32 = 255;
    pub const MaxIssuers: u32 = 2;
    pub const MaxTokensRecordFiles: u32 = 10;
}

impl pallet_loc::Config for Test {
    type LocId = u32;
    type RuntimeEvent = RuntimeEvent;
    type Hash = H256;
    type IsLegalOfficer = LoAuthorityListMock;
    type MaxMetadataItemNameSize = MaxMetadataItemNameSize;
    type MaxMetadataItemValueSize = MaxMetadataItemValueSize;
    type MaxFileNatureSize = MaxFileNatureSize;
    type MaxLinkNatureSize = MaxLinkNatureSize;
    type CollectionItemId = H256;
    type MaxCollectionItemDescriptionSize = MaxCollectionItemDescriptionSize;
    type MaxCollectionItemTokenIdSize = MaxCollectionItemTokenIdSize;
    type MaxCollectionItemTokenTypeSize = MaxCollectionItemTokenTypeSize;
    type TokensRecordId = H256;
    type MaxTokensRecordDescriptionSize = MaxTokensRecordDescriptionSize;
    type MaxFileNameSize = MaxFileNameSize;
    type MaxFileContentTypeSize = MaxFileContentTypeSize;
    type MaxTokensRecordFiles = MaxTokensRecordFiles;
    type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

pub fn new_test_ext_at_block(block_number: u64) -> sp_io::TestExternalities {
    let t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(block_number));
    ext
}
