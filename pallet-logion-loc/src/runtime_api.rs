//! Runtime API definition for LogionLoc pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use sp_api;
use codec::Codec;
use sp_runtime::traits::MaybeDisplay;
use crate::LocType;

sp_api::decl_runtime_apis! {

    pub trait FeesApi<Balance, TokenIssuance>
    where Balance: Codec + MaybeDisplay, TokenIssuance: Codec + MaybeDisplay
    {
        /// Query expected fees for submitting given files
        fn query_file_storage_fee(num_of_entries: u32, tot_size: u32) -> Balance;

        /// Query expected legal fees for opening a LOC with given type
        fn query_legal_fee(loc_type: LocType) -> Balance;

        /// Query expected item legal fees for adding an item with given type
        fn query_item_legal_fee(token_issuance: TokenIssuance) -> Balance;
    }
}