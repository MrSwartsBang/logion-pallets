#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::codec::{Decode, Encode};
use frame_support::dispatch::{DispatchResultWithPostInfo, Vec};
use frame_support::error::BadOrigin;
use frame_support::traits::EnsureOrigin;
use logion_shared::{IsLegalOfficer, LegalOfficerCreation};
use scale_info::TypeInfo;
use sp_core::OpaquePeerId as PeerId;
use sp_std::collections::btree_set::BTreeSet;

pub use pallet::*;

pub mod migrations;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
use frame_system::RawOrigin;

#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub enum LegalOfficerData<AccountId> {
    Host(HostData),
    Guest(AccountId),
}

#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub struct HostData {
    pub node_id: Option<PeerId>,
    pub base_url: Option<Vec<u8>>,
}

impl Default for HostData {

    fn default() -> Self {
        HostData {
            node_id: Option::None,
            base_url: Option::None,
        }
    }
}

#[frame_support::pallet]
pub mod pallet {
    use frame_system::pallet_prelude::*;
    use frame_support::{
        dispatch::DispatchResultWithPostInfo,
        pallet_prelude::*,
    };
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {

        /// The origin which can add a Logion Legal Officer.
        type AddOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// The origin which can remove a Logion Legal Officer.
        type RemoveOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// The origin which can update a Logion Legal Officer's data (in addition to himself).
        type UpdateOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    /// All LOs indexed by their account ID.
    #[pallet::storage]
    #[pallet::getter(fn legal_officer_set)]
    pub type LegalOfficerSet<T> = StorageMap<_, Blake2_128Concat, <T as frame_system::Config>::AccountId, LegalOfficerData<<T as frame_system::Config>::AccountId>>;

    /// The set of LO nodes.
    #[pallet::storage]
    #[pallet::getter(fn legal_officer_nodes)]
    pub type LegalOfficerNodes<T> = StorageValue<_, BTreeSet<PeerId>, ValueQuery>;

    #[derive(Encode, Decode, Eq, PartialEq, Debug, TypeInfo)]
    pub enum StorageVersion {
        V1,
        V2AddOnchainSettings,
        V3GuestLegalOfficers,
    }

    impl Default for StorageVersion {
        fn default() -> StorageVersion {
            return StorageVersion::V3GuestLegalOfficers;
        }
    }

    /// Storage version
    #[pallet::storage]
    #[pallet::getter(fn pallet_storage_version)]
    pub type PalletStorageVersion<T> = StorageValue<_, StorageVersion, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub legal_officers: Vec<(T::AccountId, LegalOfficerData<T::AccountId>)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self { legal_officers: Vec::new() }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            Pallet::<T>::initialize_legal_officers(&self.legal_officers);
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Issued when an LO is added to the list. [accountId]
        LoAdded(T::AccountId),
        /// Issued when an LO is removed from the list. [accountId]
        LoRemoved(T::AccountId),
        /// Issued when an LO is updated. [accountId]
        LoUpdated(T::AccountId),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The LO is already in the list.
        AlreadyExists,
        /// The LO is not in the list.
        NotFound,
        /// The Peer ID is already assigned to another LO.
        PeerIdAlreadyInUse,
        /// The host has at least one guest and cannot become a guest or be removed
        HostHasGuest,
        /// Trying to add a guest with another guest as host
        GuestOfGuest,
        /// Trying to add a guest with unknown host
        HostNotFound,
        /// Host cannot convert itself into a guest
        HostCannotConvert,
        /// Guest cannot update
        GuestCannotUpdate,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T:Config> Pallet<T> {

        /// Adds a new LO to the list
        #[pallet::weight(0)]
        pub fn add_legal_officer(
            origin: OriginFor<T>,
            legal_officer_id: T::AccountId,
            data: LegalOfficerData<T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            T::AddOrigin::ensure_origin(origin)?;
            Self::do_add_legal_officer(
                legal_officer_id,
                data
            )
        }

        /// Removes a LO from the list
        #[pallet::weight(0)]
        pub fn remove_legal_officer(
            origin: OriginFor<T>,
            legal_officer_id: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            T::RemoveOrigin::ensure_origin(origin)?;
            let to_remove = <LegalOfficerSet<T>>::get(&legal_officer_id);
            if to_remove.is_none() {
                Err(Error::<T>::NotFound)?
            } else if Self::host_has_guest(&legal_officer_id) {
                Err(Error::<T>::HostHasGuest)?
            } else {
                <LegalOfficerSet<T>>::remove(&legal_officer_id);
                Self::try_reset_legal_officer_nodes(&to_remove.unwrap())?;

                Self::deposit_event(Event::LoRemoved(legal_officer_id));
                Ok(().into())
            }
        }

        /// Updates an existing LO's data
        #[pallet::weight(0)]
        pub fn update_legal_officer(
            origin: OriginFor<T>,
            legal_officer_id: T::AccountId,
            data: LegalOfficerData<T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed_or_root(origin.clone())?;
            if who.is_some() && who.clone().unwrap() != legal_officer_id {
                T::UpdateOrigin::ensure_origin(origin)?;
            }
            let to_update = <LegalOfficerSet<T>>::get(&legal_officer_id);
            if to_update.is_none() {
                Err(Error::<T>::NotFound)?
            } else {
                Self::ensure_host_if_guest(&data)?;
                let some_to_update = to_update.unwrap();
                match some_to_update {
                    LegalOfficerData::Host(_) => match data {
                        LegalOfficerData::Guest(_) => {
                            if Self::host_has_guest(&legal_officer_id) {
                                Err(Error::<T>::HostHasGuest)?
                            }
                            if who.is_some() && who.unwrap() == legal_officer_id {
                                Err(Error::<T>::HostCannotConvert)?
                            }
                        },
                        _ => (),
                    },
                    LegalOfficerData::Guest(_) => if who.is_some() && who.unwrap() == legal_officer_id {
                        Err(Error::<T>::GuestCannotUpdate)?
                    },
                }

                <LegalOfficerSet<T>>::set(legal_officer_id.clone(), Some(data.clone()));
                match some_to_update {
                    LegalOfficerData::Guest(_) => match data {
                        LegalOfficerData::Host(_) => Self::reset_legal_officer_nodes()?,
                        LegalOfficerData::Guest(_) => (),
                    },
                    LegalOfficerData::Host(_) => {
                        Self::reset_legal_officer_nodes()?
                    },
                }

                Self::deposit_event(Event::LoUpdated(legal_officer_id));
                Ok(().into())
            }
        }
    }
}

pub type OuterOrigin<T> = <T as frame_system::Config>::RuntimeOrigin;

impl<T: Config> Pallet<T> {
    fn initialize_legal_officers(legal_officers: &Vec<(T::AccountId, LegalOfficerData<T::AccountId>)>) {
        for legal_officer in legal_officers {
            LegalOfficerSet::<T>::insert::<&T::AccountId, &LegalOfficerData<T::AccountId>>(&(legal_officer.0), &(legal_officer.1));
            LegalOfficerNodes::<T>::set(BTreeSet::new());
        }
    }

    fn try_reset_legal_officer_nodes(added_or_removed_data: &LegalOfficerData<T::AccountId>) -> Result<(), Error<T>> {
        match added_or_removed_data {
            LegalOfficerData::Host(_) => Self::reset_legal_officer_nodes(),
            _ => Ok(()),
        }
    }

    fn reset_legal_officer_nodes() -> Result<(), Error<T>> {
        let mut new_nodes = BTreeSet::new();
        for data in LegalOfficerSet::<T>::iter_values() {
            match data {
                LegalOfficerData::Host(host_data) => {
                    if host_data.node_id.is_some() && ! new_nodes.insert(host_data.node_id.unwrap()) {
                        Err(Error::<T>::PeerIdAlreadyInUse)?
                    }
                },
                _ => (),
            }
        }
        LegalOfficerNodes::<T>::set(new_nodes);
        Ok(())
    }

    pub fn ensure_legal_officer(o: T::RuntimeOrigin) -> Result<T::AccountId, BadOrigin> {
        <Self as EnsureOrigin<T::RuntimeOrigin>>::ensure_origin(o)
    }

    fn host_has_guest(host_id: &T::AccountId) -> bool {
        for data in LegalOfficerSet::<T>::iter_values() {
            match data {
                LegalOfficerData::Guest(host) =>
                    if host == *host_id { return true },
                _ => (),
            }
        }
        false
    }

    fn ensure_host_if_guest(data: &LegalOfficerData<T::AccountId>) -> Result<(), Error<T>> {
        match &data {
            LegalOfficerData::Guest(host) => Self::ensure_host(host),
            _ => Ok(()),
        }
    }

    fn ensure_host(id: &T::AccountId) -> Result<(), Error<T>> {
        let potential_host = LegalOfficerSet::<T>::get(id);
        if potential_host.is_none() {
            Err(Error::<T>::HostNotFound)
        } else {
            match potential_host.unwrap() {
                LegalOfficerData::Guest(_) => Err(Error::<T>::GuestOfGuest),
                LegalOfficerData::Host(_) => Ok(()),
            }
        }
    }

    fn do_add_legal_officer(
        legal_officer_id: T::AccountId,
        data: LegalOfficerData<T::AccountId>,
    ) -> DispatchResultWithPostInfo {
        if <LegalOfficerSet<T>>::contains_key(&legal_officer_id) {
            Err(Error::<T>::AlreadyExists)?
        } else {
            Self::ensure_host_if_guest(&data)?;
            <LegalOfficerSet<T>>::insert(legal_officer_id.clone(), &data);
            Self::try_reset_legal_officer_nodes(&data)?;

            Self::deposit_event(Event::LoAdded(legal_officer_id));
            Ok(().into())
        }
    }
}

impl<T: Config> EnsureOrigin<T::RuntimeOrigin> for Pallet<T> {
    type Success = T::AccountId;

    fn try_origin(o: T::RuntimeOrigin) -> Result<Self::Success, T::RuntimeOrigin> {
        <Self as IsLegalOfficer<T::AccountId, T::RuntimeOrigin>>::try_origin(o)
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn successful_origin() -> OuterOrigin<T> {
        let first_member = match <LegalOfficerSet<T>>::iter().next() {
            Some(pair) => pair.0.clone(),
            None => Default::default(),
        };
        OuterOrigin::<T>::from(RawOrigin::Signed(first_member.clone()))
    }
}

impl<T: Config> IsLegalOfficer<T::AccountId, T::RuntimeOrigin> for Pallet<T> {

    fn is_legal_officer(account: &T::AccountId) -> bool {
        LegalOfficerSet::<T>::contains_key(account)
    }

    fn legal_officers() -> Vec<T::AccountId> {
        LegalOfficerSet::<T>::iter_keys().collect()
    }
}

impl<T: Config> LegalOfficerCreation<T::AccountId> for Pallet<T> {

    fn add_guest_legal_officer(
        guest_legal_officer_id: T::AccountId,
        host_legal_officer_id: T::AccountId) -> DispatchResultWithPostInfo {

        Pallet::<T>::do_add_legal_officer(guest_legal_officer_id, LegalOfficerData::Guest(host_legal_officer_id))
    }
}