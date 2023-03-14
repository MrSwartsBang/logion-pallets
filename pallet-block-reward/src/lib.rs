#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use frame_support::pallet_prelude::*;
use frame_support::traits::{Currency, Get, Imbalance};
use frame_system::pallet_prelude::*;
use sp_runtime::Percent;
use sp_std::vec;

#[cfg(any(feature = "runtime-benchmarks"))]
pub mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {

    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    pub(crate) type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    pub(crate) type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::NegativeImbalance;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The currency trait.
        type Currency: Currency<Self::AccountId>;

        /// Used to payout rewards
        type RewardDistributor: RewardDistributor<NegativeImbalanceOf<Self>>;

        /// The amount of issuance for each block.
        #[pallet::constant]
        type RewardAmount: Get<BalanceOf<Self>>;

        /// The reward distribution key
        type DistributionKey: Get<DistributionKey>;
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {

        fn on_finalize(_n: BlockNumberFor<T>) {
            let reward = T::Currency::issue(T::RewardAmount::get());
            Self::distribute(reward);
        }

        fn integrity_test() {
            assert!(T::DistributionKey::get().is_valid());
        }
    }

    impl<T: Config> Pallet<T> {

        fn distribute(block_reward: NegativeImbalanceOf<T>) {
            let block_reward_balance = block_reward.peek();
            let distribution_key = T::DistributionKey::get();

            let stakers_part = distribution_key.stakers_percent * block_reward_balance;
            let collators_part = distribution_key.collators_percent * block_reward_balance;

            let (stakers_imbalance, remainder) = block_reward.split(stakers_part);
            let (collators_imbalance, reserve_imbalance) = remainder.split(collators_part);

            T::RewardDistributor::payout_stakers(stakers_imbalance);
            T::RewardDistributor::payout_reserve(reserve_imbalance);
            T::RewardDistributor::payout_collators(collators_imbalance);
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct DistributionKey {
    pub reserve_percent: Percent,
    pub stakers_percent: Percent,
    pub collators_percent: Percent,
}

impl DistributionKey {

    fn is_valid(&self) -> bool {
        let mut should_become_zero = Self::into_signed(Percent::one());

        should_become_zero = should_become_zero - Self::into_signed(self.reserve_percent);
        should_become_zero = should_become_zero - Self::into_signed(self.stakers_percent);
        should_become_zero = should_become_zero - Self::into_signed(self.collators_percent);

        should_become_zero == 0
    }

    fn into_signed(percent: Percent) -> i16 {
        <u8 as Into<i16>>::into(percent.deconstruct())
    }
}

pub trait RewardDistributor<Imbalance> {

    fn payout_collators(reward: Imbalance);

    fn payout_reserve(reward: Imbalance);

    fn payout_stakers(reward: Imbalance);
}
