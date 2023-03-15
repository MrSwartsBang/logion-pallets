use crate::{self as pallet_block_reward, NegativeImbalanceOf};

use frame_support::{
    construct_runtime, parameter_types, traits::Currency,
};

use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup}, Percent,
};
use logion_shared::{DistributionKey, RewardDistributor};

pub type AccountId = u64;
pub type BlockNumber = u64;
pub type Balance = u128;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
    pub struct Test
    where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Balances: pallet_balances,
        BlockReward: pallet_block_reward,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type RuntimeOrigin = RuntimeOrigin;
    type Index = u64;
    type RuntimeCall = RuntimeCall;
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub const MaxLocks: u32 = 4;
    pub const ExistentialDeposit: Balance = 2;
}

impl pallet_balances::Config for Test {
    type MaxLocks = MaxLocks;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
}

// Fake accounts used to simulate reward beneficiaries balances
pub const RESERVE_ACCOUNT: AccountId = 2;
pub const COLLATORS_ACCOUNT: AccountId = 3;
pub const STAKERS_ACCOUNT: AccountId = 4;

// Type used as beneficiary payout handle
pub struct RewardDistributorImpl();
impl pallet_block_reward::RewardDistributor<NegativeImbalanceOf<Test>>
    for RewardDistributorImpl
{
    fn payout_reserve(reward: NegativeImbalanceOf<Test>) {
        Balances::resolve_creating(&RESERVE_ACCOUNT, reward);
    }

    fn payout_collators(reward: NegativeImbalanceOf<Test>) {
        Balances::resolve_creating(&COLLATORS_ACCOUNT, reward);
    }

    fn payout_stakers(reward: NegativeImbalanceOf<Test>) {
        Balances::resolve_creating(&STAKERS_ACCOUNT, reward);
    }
}

pub const BLOCK_REWARD: Balance = 10_000_000_000_000_000_000; // 10 LGNT

parameter_types! {
    pub const RewardAmount: Balance = BLOCK_REWARD;
    pub const RewardDistributionKey: DistributionKey = DistributionKey {
        stakers_percent: Percent::from_percent(50),
        collators_percent: Percent::from_percent(30),
        reserve_percent: Percent::from_percent(20),
    };
}

impl pallet_block_reward::Config for Test {
    type Currency = Balances;
    type RewardAmount = RewardAmount;
    type RewardDistributor = RewardDistributorImpl;
    type DistributionKey = RewardDistributionKey;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
