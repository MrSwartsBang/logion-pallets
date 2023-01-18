#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    dispatch::{GetDispatchInfo, Vec, Weight},
    Parameter,
    traits::{EnsureOrigin, UnfilteredDispatchable},
};
use frame_support::dispatch::DispatchResultWithPostInfo;
use frame_system::{ensure_signed, RawOrigin};
use sp_std::boxed::Box;

pub trait CreateRecoveryCallFactory<Origin, AccountId, BlockNumber> {
    type Call: Parameter + UnfilteredDispatchable<RuntimeOrigin = Origin> + GetDispatchInfo;

    fn build_create_recovery_call(legal_officers: Vec<AccountId>, threshold: u16, delay_period: BlockNumber) -> Self::Call;
}

pub struct LegalOfficerCaseSummary<AccountId> {
    pub owner: AccountId,
    pub requester: Option<AccountId>,
}

pub trait LocQuery<LocId, AccountId> {
    fn has_closed_identity_locs(account: &AccountId, legal_officer: &Vec<AccountId>) -> bool;
    fn get_loc(loc_id: &LocId) -> Option<LegalOfficerCaseSummary<AccountId>>;
}

pub trait LocValidity<LocId, AccountId> {
    fn loc_valid_with_owner(loc_id: &LocId, legal_officer: &AccountId) -> bool;
}

pub trait MultisigApproveAsMultiCallFactory<Origin, AccountId, Timepoint> {
    type Call: Parameter + UnfilteredDispatchable<RuntimeOrigin = Origin> + GetDispatchInfo;

    fn build_approve_as_multi_call(
        threshold: u16,
        other_signatories: Vec<AccountId>,
        maybe_timepoint: Option<Timepoint>,
        call_hash: [u8; 32],
        max_weight: Weight,
    ) -> Self::Call;
}

pub trait MultisigAsMultiCallFactory<Origin, AccountId, Timepoint> {
    type Call: Parameter + UnfilteredDispatchable<RuntimeOrigin = Origin> + GetDispatchInfo;

    fn build_as_multi_call(
        threshold: u16,
        other_signatories: Vec<AccountId>,
        maybe_timepoint: Option<Timepoint>,
        call: Box<Self::Call>,
        max_weight: Weight,
    ) -> Self::Call;
}

pub trait IsLegalOfficer<AccountId: PartialEq, Origin: Clone + Into<Result<RawOrigin<AccountId>, Origin>>>: EnsureOrigin<Origin, Success = AccountId> {
    fn is_legal_officer(account: &AccountId) -> bool {
        Self::legal_officers().contains(account)
    }

    fn try_origin(o: Origin) -> Result<AccountId, Origin> {
        let result = ensure_signed(o.clone());
        match result {
            Ok(who) => {
                if Self::is_legal_officer(&who) {
                    Ok(who)
                } else {
                    Err(o)
                }
            },
            Err(_) => Err(o)
        }
    }

    fn legal_officers() -> Vec<AccountId>;
}

pub trait LegalOfficerCreation<AccountId> {
    fn add_guest_legal_officer(
        guest_legal_officer_id: AccountId,
        host_legal_officer_id: AccountId,
    ) -> DispatchResultWithPostInfo;
}
