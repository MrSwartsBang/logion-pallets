#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::codec::{Decode, Encode};
use frame_support::dispatch::Vec;
use frame_support::error::BadOrigin;
use frame_support::traits::EnsureOrigin;
use logion_shared::IsLegalOfficer;
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
pub struct LegalOfficerData {
	pub node_id: Option<PeerId>,
	pub base_url: Option<Vec<u8>>,
}

impl Default for LegalOfficerData {

	fn default() -> Self {
		LegalOfficerData {
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
	pub type LegalOfficerSet<T> = StorageMap<_, Blake2_128Concat, <T as frame_system::Config>::AccountId, LegalOfficerData>;

	/// The set of LO nodes.
	#[pallet::storage]
	#[pallet::getter(fn legal_officer_nodes)]
	pub type LegalOfficerNodes<T> = StorageValue<_, BTreeSet<PeerId>, ValueQuery>;

	#[derive(Encode, Decode, Eq, PartialEq, Debug, TypeInfo)]
	pub enum StorageVersion {
		V1,
		V2AddOnchainSettings,
	}

	impl Default for StorageVersion {
		fn default() -> StorageVersion {
			return StorageVersion::V1;
		}
	}

	/// Storage version
	#[pallet::storage]
	#[pallet::getter(fn pallet_storage_version)]
	pub type PalletStorageVersion<T> = StorageValue<_, StorageVersion, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub legal_officers: Vec<(T::AccountId, LegalOfficerData)>,
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
			data: LegalOfficerData,
		) -> DispatchResultWithPostInfo {
			T::AddOrigin::ensure_origin(origin)?;
			if <LegalOfficerSet<T>>::contains_key(&legal_officer_id) {
				Err(Error::<T>::AlreadyExists)?
			} else {
				<LegalOfficerSet<T>>::insert(legal_officer_id.clone(), data);
				Self::reset_legal_officer_nodes()?;

				Self::deposit_event(Event::LoAdded(legal_officer_id));
				Ok(().into())
			}
		}

		/// Removes a LO from the list
		#[pallet::weight(0)]
		pub fn remove_legal_officer(
			origin: OriginFor<T>,
			legal_officer_id: T::AccountId,
		) -> DispatchResultWithPostInfo {
			T::RemoveOrigin::ensure_origin(origin)?;
			if ! <LegalOfficerSet<T>>::contains_key(&legal_officer_id) {
				Err(Error::<T>::NotFound)?
			} else {
				<LegalOfficerSet<T>>::remove(&legal_officer_id);
				Self::reset_legal_officer_nodes()?;

				Self::deposit_event(Event::LoRemoved(legal_officer_id));
				Ok(().into())
			}
		}

		/// Updates an existing LO's data
		#[pallet::weight(0)]
		pub fn update_legal_officer(
			origin: OriginFor<T>,
			legal_officer_id: T::AccountId,
			data: LegalOfficerData,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed_or_root(origin.clone())?;
			if who.is_some() && who.unwrap() != legal_officer_id {
				T::UpdateOrigin::ensure_origin(origin)?;
			}
			if ! <LegalOfficerSet<T>>::contains_key(&legal_officer_id) {
				Err(Error::<T>::NotFound)?
			} else {
				<LegalOfficerSet<T>>::set(legal_officer_id.clone(), Some(data));
				Self::reset_legal_officer_nodes()?;

				Self::deposit_event(Event::LoUpdated(legal_officer_id));
				Ok(().into())
			}
		}
	}
}

pub type OuterOrigin<T> = <T as frame_system::Config>::RuntimeOrigin;

impl<T: Config> Pallet<T> {
	fn initialize_legal_officers(legal_officers: &Vec<(T::AccountId, LegalOfficerData)>) {
		for legal_officer in legal_officers {
			LegalOfficerSet::<T>::insert::<&T::AccountId, &LegalOfficerData>(&(legal_officer.0), &(legal_officer.1));
			LegalOfficerNodes::<T>::set(BTreeSet::new());
		}
	}

	fn reset_legal_officer_nodes() -> Result<(), Error<T>> {
		let mut new_nodes = BTreeSet::new();
		for data in LegalOfficerSet::<T>::iter_values() {
			if data.node_id.is_some() && ! new_nodes.insert(data.node_id.unwrap()) {
				Err(Error::<T>::PeerIdAlreadyInUse)?
			}
		}
		LegalOfficerNodes::<T>::set(new_nodes);
		Ok(())
	}

	pub fn ensure_legal_officer(o: T::RuntimeOrigin) -> Result<T::AccountId, BadOrigin> {
		<Self as EnsureOrigin<T::RuntimeOrigin>>::ensure_origin(o)
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
}
