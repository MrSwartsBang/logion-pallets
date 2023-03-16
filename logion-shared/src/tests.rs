use frame_support::sp_runtime::Percent;
use crate::DistributionKey;

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

