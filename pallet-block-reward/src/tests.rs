use super::*;
use mock::*;

#[test]
pub fn inflation_as_expected() {
    new_test_ext().execute_with(|| {
        for block in 0..10 {
            assert_eq!(
                <Test as Config>::Currency::total_issuance(),
                block * BLOCK_REWARD
            );
            BlockReward::on_finalize(block.try_into().unwrap());
            assert_eq!(
                <Test as Config>::Currency::total_issuance(),
                (block + 1) * BLOCK_REWARD
            );
        }
    })
}

#[test]
pub fn reward_distributed_as_expected() {
    new_test_ext().execute_with(|| {
        BlockReward::on_finalize(0);

        assert_eq!(get_free_balance(RESERVE_ACCOUNT), 2_000_000_000_000_000_000);
        assert_eq!(get_free_balance(STAKERS_ACCOUNT), 5_000_000_000_000_000_000);
        assert_eq!(get_free_balance(COLLATORS_ACCOUNT), 3_000_000_000_000_000_000);
    })
}

fn get_free_balance(account_id: AccountId) -> Balance {
    <Test as Config>::Currency::free_balance(account_id)
}
