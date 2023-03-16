#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use frame_support::pallet_prelude::*;
use frame_support::traits::{Currency, Get};
use frame_system::pallet_prelude::*;
use sp_std::vec;

#[cfg(any(feature = "runtime-benchmarks"))]
pub mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use logion_shared::{DistributionKey, RewardDistributor};
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
        type RewardDistributor: RewardDistributor<NegativeImbalanceOf<Self>, BalanceOf<Self>>;

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

            T::RewardDistributor::distribute(block_reward, T::DistributionKey::get());
        }
    }
}
