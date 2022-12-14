use frame_support::{assert_err, assert_ok};
use sp_runtime::DispatchError::BadOrigin;
use crate::mock::*;
use crate::{Error, Vote};

const WRONG_LOC_ID: u32 = 2;
const WALLET_USER: u64 = 2;

#[test]
fn it_creates_vote() {
    new_test_ext().execute_with(|| {
        assert!(LogionVote::votes().is_empty());
        assert_ok!(LogionVote::create_vote(RuntimeOrigin::signed(LEGAL_OFFICER1), LOC_ID));
        let votes = LogionVote::votes();
        assert!(votes.len() == 1);
        assert_eq!(votes[0], Vote { loc_id: LOC_ID })
    });
}

#[test]
fn it_fails_to_create_vote_when_not_legal_officer() {
    new_test_ext().execute_with(|| {
        assert!(LogionVote::votes().is_empty());
        assert_err!(LogionVote::create_vote(RuntimeOrigin::signed(WALLET_USER), LOC_ID), BadOrigin);
        assert!(LogionVote::votes().is_empty());
    });
}

#[test]
fn it_fails_to_create_vote_when_wrong_loc() {
    new_test_ext().execute_with(|| {
        assert!(LogionVote::votes().is_empty());
        assert_err!(LogionVote::create_vote(RuntimeOrigin::signed(LEGAL_OFFICER1), WRONG_LOC_ID), Error::<Test>::InvalidLoc);
        assert!(LogionVote::votes().is_empty());
    });
}
