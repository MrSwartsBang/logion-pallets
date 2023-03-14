use super::*;
use mock::*;
use sp_runtime::Percent;

#[test]
fn distribution_key_with_only_reserve_is_valid() {
    let key = DistributionKey {
        reserve_percent: Percent::from_percent(100),
        stakers_percent: Percent::from_percent(0),
        collators_percent: Percent::from_percent(0),
    };
    assert!(key.is_valid());
}

#[test]
fn distribution_key_with_only_stakers_is_valid() {
    let key = DistributionKey {
        reserve_percent: Percent::from_percent(0),
        stakers_percent: Percent::from_percent(100),
        collators_percent: Percent::from_percent(0),
    };
    assert!(key.is_valid());
}

#[test]
fn distribution_key_with_only_collators_is_valid() {
    let key = DistributionKey {
        reserve_percent: Percent::from_percent(0),
        stakers_percent: Percent::from_percent(0),
        collators_percent: Percent::from_percent(100),
    };
    assert!(key.is_valid());
}

#[test]
fn distribution_key_is_valid() {
    let key = DistributionKey {
        reserve_percent: Percent::from_percent(50),
        stakers_percent: Percent::from_percent(30),
        collators_percent: Percent::from_percent(20),
    };
    assert!(key.is_valid());
}

#[test]
fn distribution_key_invalid_lower_than_hundred() {
    let key = DistributionKey {
        reserve_percent: Percent::from_percent(49),
        stakers_percent: Percent::from_percent(30),
        collators_percent: Percent::from_percent(20),
    };
    assert!(!key.is_valid());
}

#[test]
fn distribution_key_invalid_greater_than_hundred() {
    let key = DistributionKey {
        reserve_percent: Percent::from_percent(51),
        stakers_percent: Percent::from_percent(30),
        collators_percent: Percent::from_percent(20),
    };
    assert!(!key.is_valid());
}

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
