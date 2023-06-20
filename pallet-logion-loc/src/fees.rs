use logion_shared::Beneficiary;

use crate::{
    Config,
    mock::*
};

/// Balances of accounts involved in fees payment
pub struct BalancesSnapshot {
    pub payer_account: AccountId,
    pub legal_officer_account: AccountId,
    pub payer: Balance,
    pub treasury: Balance,
    pub stakers: Balance,
    pub collators: Balance,
    pub reserve: Balance,
    pub legal_officer: Balance,
}
impl BalancesSnapshot {

    pub fn take(payer_account: AccountId, legal_officer_account: AccountId) -> BalancesSnapshot {
        BalancesSnapshot {
            payer_account,
            legal_officer_account,
            payer: Self::get_free_balance(payer_account),
            treasury: Self::get_free_balance(TREASURY_ACCOUNT_ID),
            stakers: Self::get_free_balance(STAKERS_ACCOUNT),
            collators: Self::get_free_balance(COLLATORS_ACCOUNT),
            reserve: Self::get_free_balance(RESERVE_ACCOUNT),
            legal_officer: Self::get_free_balance(legal_officer_account),
        }
    }

    fn get_free_balance(account_id: AccountId) -> Balance {
        <Test as Config>::Currency::free_balance(account_id)
    }

    pub fn delta_since(&self, previous: &BalancesSnapshot) -> BalancesDelta {
        BalancesDelta {
            payer: previous.payer.saturating_sub(Self::get_free_balance(previous.payer_account)),
            treasury: Self::get_free_balance(TREASURY_ACCOUNT_ID).saturating_sub(previous.treasury),
            stakers: Self::get_free_balance(STAKERS_ACCOUNT).saturating_sub(previous.stakers),
            collators: Self::get_free_balance(COLLATORS_ACCOUNT).saturating_sub(previous.collators),
            reserve: Self::get_free_balance(RESERVE_ACCOUNT).saturating_sub(previous.reserve),
            legal_officer: Self::get_free_balance(previous.legal_officer_account).saturating_sub(previous.legal_officer),
        }
    }
}

/// Amounts that were credited or debited between 2 snapshots
pub struct BalancesDelta {
    /// Debited amount or 0 if credited
    payer: u128,
    /// Credited amount or 0 if debited
    treasury: u128,
    /// Credited amount or 0 if debited
    stakers: u128,
    /// Credited amount or 0 if debited
    collators: u128,
    /// Credited amount or 0 if debited
    reserve: u128,
    /// Credited amount or 0 if debited
    legal_officer: u128,
}
impl BalancesDelta {

    pub fn total_credited(&self) -> u128 {
        self.treasury + self.stakers + self.collators + self.reserve + self.legal_officer
    }

    pub fn total_debited(&self) -> u128 {
        self.payer
    }
}

/// Other fees than inclusion
pub struct Fees {
    pub storage_fees: Balance,
    pub legal_fees: Balance,
    /// When legal_fees is > 0, legal_fee_beneficiary must be some; should be none otherwise
    pub legal_fee_beneficiary: Option<Beneficiary<AccountId>>,
    pub certificate_fees: Balance,
}
impl Fees {

    pub fn total(&self) -> Balance {
        self.storage_fees + self.legal_fees + self.certificate_fees
    }

    pub fn only_storage(num_of_files: u32, tot_size: u32) -> Fees {
        Fees {
            certificate_fees: 0,
            legal_fees: 0,
            storage_fees: Self::storage_fees(num_of_files, tot_size),
            legal_fee_beneficiary: None,
        }
    }

    pub fn storage_fees(num_of_files: u32, tot_size: u32) -> Balance {
        let entry_fee: Balance = Into::<Balance>::into(num_of_files) * Into::<Balance>::into(FileStorageEntryFee::get());
        let storage_fee: Balance = Into::<Balance>::into(tot_size) * Into::<Balance>::into(FileStorageByteFee::get());
        entry_fee + storage_fee
    }

    pub fn only_legal(fee: Balance, beneficiary: Beneficiary<AccountId>) -> Fees {
        Fees {
            certificate_fees: 0,
            legal_fees: fee,
            storage_fees: 0,
            legal_fee_beneficiary: Some(beneficiary),
        }
    }

    pub fn assert_balances_events(&self, previous_balances: BalancesSnapshot) {
        let expected_fees_total = self.total();
    
        let current_balances = BalancesSnapshot::take(previous_balances.payer_account, previous_balances.legal_officer_account);
        let balances_delta = current_balances.delta_since(&previous_balances);
        let credited_fees: Balance = balances_delta.total_credited();
        assert_eq!(credited_fees, expected_fees_total);
    
        let debited_fees = balances_delta.total_debited();
        assert_eq!(debited_fees, expected_fees_total);
    
        if self.storage_fees > 0 {
            System::assert_has_event(RuntimeEvent::LogionLoc(crate::Event::StorageFeeWithdrawn {
                0: previous_balances.payer_account,
                1: self.storage_fees,
            }));
        }
    
        if self.legal_fees > 0 {
            System::assert_has_event(RuntimeEvent::LogionLoc(crate::Event::LegalFeeWithdrawn {
                0: previous_balances.payer_account,
                1: self.legal_fee_beneficiary.unwrap(),
                2: self.legal_fees,
            }));
        }
    
        if self.certificate_fees > 0 {
            System::assert_has_event(RuntimeEvent::LogionLoc(crate::Event::CertificateFeeWithdrawn {
                0: previous_balances.payer_account,
                1: self.certificate_fees,
            }));
        }
    }
    
}
