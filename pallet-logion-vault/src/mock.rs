use crate::{self as pallet_logion_vault};
use logion_shared::{IsLegalOfficer, MultisigApproveAsMultiCallFactory, MultisigAsMultiCallFactory};
use pallet_multisig::Timepoint;
use sp_core::hash::H256;
use frame_support::{parameter_types, dispatch::Weight, traits::EnsureOrigin};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup}, testing::Header,
};
use frame_system as system;
use sp_std::convert::{TryInto, TryFrom};
use system::ensure_signed;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Vault: pallet_logion_vault::{Pallet, Call, Storage, Event<T>},
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

pub struct CreateRecoveryCallFactoryMock;
impl MultisigApproveAsMultiCallFactory<<Test as system::Config>::RuntimeOrigin, <Test as system::Config>::AccountId, Timepoint<<Test as system::Config>::BlockNumber>> for CreateRecoveryCallFactoryMock {
    type Call = RuntimeCall;

    fn build_approve_as_multi_call(
		_threshold: u16,
        _other_signatories: Vec<<Test as system::Config>::AccountId>,
        _maybe_timepoint: Option<Timepoint<<Test as system::Config>::BlockNumber>>,
        _call_hash: [u8; 32],
        _max_weight: Weight
	) -> Self::Call {
        RuntimeCall::System(frame_system::Call::remark{ remark : Vec::from([0u8]) })
    }
}

pub struct MultisigAsMultiCallFactoryMock;
impl MultisigAsMultiCallFactory<<Test as system::Config>::RuntimeOrigin, <Test as system::Config>::AccountId, Timepoint<<Test as system::Config>::BlockNumber>> for MultisigAsMultiCallFactoryMock {
    type Call = RuntimeCall;

    fn build_as_multi_call(
		_threshold: u16,
        _other_signatories: Vec<<Test as system::Config>::AccountId>,
        _maybe_timepoint: Option<Timepoint<<Test as system::Config>::BlockNumber>>,
        _call: Box<Self::Call>,
        _max_weight: Weight,
	) -> Self::Call {
        *_call
    }
}

pub const LEGAL_OFFICER1: u64 = 1;
pub const LEGAL_OFFICER2: u64 = 2;
pub const USER_ID: u64 = 3;
pub const ANOTHER_USER_ID: u64 = 4;

pub struct IsLegalOfficerMock;
impl IsLegalOfficer<<Test as system::Config>::AccountId, RuntimeOrigin> for IsLegalOfficerMock {

	fn legal_officers() -> Vec<<Test as system::Config>::AccountId> {
		vec![LEGAL_OFFICER1, LEGAL_OFFICER2]
	}
}

impl EnsureOrigin<RuntimeOrigin> for IsLegalOfficerMock {
    type Success = <Test as system::Config>::AccountId;

    fn try_origin(o: RuntimeOrigin) -> std::result::Result<Self::Success, RuntimeOrigin> {
		let result = ensure_signed(o.clone());
        match result {
			Ok(who) => {
				if <Self as IsLegalOfficer<Self::Success, RuntimeOrigin>>::is_legal_officer(&who) {
					Ok(who)
				} else {
					Err(o)
				}
			},
			Err(_) => Err(o)
		}
    }
}

impl pallet_logion_vault::Config for Test {
	type RuntimeCall = RuntimeCall;
	type MultisigApproveAsMultiCallFactory = CreateRecoveryCallFactoryMock;
	type MultisigAsMultiCallFactory = MultisigAsMultiCallFactoryMock;
	type IsLegalOfficer = IsLegalOfficerMock;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
