use frame_support::{assert_err, assert_ok};
use sp_runtime::DispatchError::BadOrigin;
use crate::mock::*;
use crate::{Ballot, BallotStatus, Error, Vote};

const WRONG_LOC_ID: u32 = 2;
const WALLET_USER: u64 = 2;

#[test]
fn it_creates_vote() {
    new_test_ext().execute_with(|| {
        assert_empty_storage();
        assert_ok!(LogionVote::create_vote_for_all_legal_officers(RuntimeOrigin::signed(LEGAL_OFFICER1), LOC_ID));
        assert_eq!(LogionVote::last_vote_id(), 1);
        assert_eq!(LogionVote::votes(1), Some(
            Vote {
                loc_id: LOC_ID,
                ballots: vec![ Ballot { voter: LEGAL_OFFICER1, status: BallotStatus::NotVoted }]
            }))
    });
}

#[test]
fn it_fails_to_create_vote_when_not_legal_officer() {
    new_test_ext().execute_with(|| {
        assert_empty_storage();
        assert_err!(LogionVote::create_vote_for_all_legal_officers(RuntimeOrigin::signed(WALLET_USER), LOC_ID), BadOrigin);
        assert_empty_storage();
    });
}

#[test]
fn it_fails_to_create_vote_when_wrong_loc() {
    new_test_ext().execute_with(|| {
        assert_empty_storage();
        assert_err!(LogionVote::create_vote_for_all_legal_officers(RuntimeOrigin::signed(LEGAL_OFFICER1), WRONG_LOC_ID), Error::<Test>::InvalidLoc);
        assert_empty_storage();
    });
}

fn assert_empty_storage() {
    assert_eq!(LogionVote::votes(1), None);
    assert_eq!(LogionVote::last_vote_id(), 0);
}
