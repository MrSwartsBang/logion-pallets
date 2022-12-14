#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::codec::{Decode, Encode};
use frame_support::dispatch::Vec;
use scale_info::TypeInfo;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub struct Vote<LocId> {
    loc_id: LocId
}

#[frame_support::pallet]
pub mod pallet {
    use codec::HasCompact;
    use frame_support::{
        dispatch::DispatchResultWithPostInfo,
        pallet_prelude::*,
    };
    use frame_system::pallet_prelude::OriginFor;
    use logion_shared::{IsLegalOfficer, LocValidity};
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// LOC identifier
        type LocId: Member + Parameter + Default + Copy + HasCompact;

        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Query for checking that a signer is a legal officer
        type IsLegalOfficer: IsLegalOfficer<Self::AccountId, Self::RuntimeOrigin>;

        /// Query for checking the existence of a closed Identity LOC
        type LocValidity: LocValidity<Self::LocId, Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// Votes
    #[pallet::storage]
    #[pallet::getter(fn votes)]
    pub type Votes<T> = StorageValue<_, Vec<Vote<<T as Config>::LocId>>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Issued upon new Vote creation. [voteId]
        VoteCreated(u64)
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Given LOC is not valid (not found, or not closed or void) or does not belong to vote requester.
        InvalidLoc,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {

        /// Creates a new Vote.
        #[pallet::weight(0)]
        pub fn create_vote(
            origin: OriginFor<T>,
            #[pallet::compact] loc_id: T::LocId
        ) -> DispatchResultWithPostInfo {
            let who = T::IsLegalOfficer::ensure_origin(origin.clone())?;
            if T::LocValidity::loc_valid_with_owner(&loc_id, &who) {
                <Votes<T>>::mutate(|votes| {
                    votes.push(Vote { loc_id });
                });
                let next_vote_id = <Votes<T>>::get().len() as u64;
                Self::deposit_event(Event::VoteCreated(next_vote_id - 1));
                Ok(().into())
            } else {
                Err(Error::<T>::InvalidLoc)?
            }
        }
    }
}
