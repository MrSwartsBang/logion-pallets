use frame_support::{assert_err, assert_ok};
use sp_runtime::DispatchError::BadOrigin;
use crate::mock::*;
use crate::{Ballot, BallotStatus, Error, Vote};

const WRONG_LOC_ID: u32 = 2;
const WALLET_USER: u64 = 100;

#[test]
fn it_creates_vote() {
    new_test_ext().execute_with(|| {
        assert_empty_storage();
        assert_ok!(LogionVote::create_vote_for_all_legal_officers(RuntimeOrigin::signed(HOST_LEGAL_OFFICER), LOC_ID));
        let vote_id = LogionVote::last_vote_id();
        assert_eq!(vote_id, 1);
        assert_eq!(LogionVote::votes(1), Some(
            Vote {
                loc_id: LOC_ID,
                ballots: vec![
                    Ballot { voter: HOST_LEGAL_OFFICER, status: BallotStatus::NotVoted },
                    Ballot { voter: LEGAL_OFFICER2, status: BallotStatus::NotVoted },
                ]
            }));
        assert_eq!(LogionVote::votes(2), None);
        assert_eq!(LogionVote::is_vote_closed_and_approved(vote_id), (false, false))
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
        assert_err!(LogionVote::create_vote_for_all_legal_officers(RuntimeOrigin::signed(HOST_LEGAL_OFFICER), WRONG_LOC_ID), Error::<Test>::InvalidLoc);
        assert_empty_storage();
    });
}

#[test]
fn it_votes_yes() {
    new_test_ext().execute_with(|| {
        assert_empty_storage();
        assert_ok!(LogionVote::create_vote_for_all_legal_officers(RuntimeOrigin::signed(HOST_LEGAL_OFFICER), LOC_ID));
        let vote_id: u64 = LogionVote::last_vote_id();
        assert_ok!(LogionVote::vote(RuntimeOrigin::signed(HOST_LEGAL_OFFICER), vote_id, true));
        assert_eq!(LogionVote::votes(vote_id), Some(
            Vote {
                loc_id: LOC_ID,
                ballots: vec![
                    Ballot { voter: HOST_LEGAL_OFFICER, status: BallotStatus::VotedYes },
                    Ballot { voter: LEGAL_OFFICER2, status: BallotStatus::NotVoted },
                ]
            }));
        assert_eq!(LogionVote::is_vote_closed_and_approved(vote_id), (false, false))
    });
}

#[test]
fn it_votes_yes_and_no() {
    new_test_ext().execute_with(|| {
        assert_empty_storage();
        assert_ok!(LogionVote::create_vote_for_all_legal_officers(RuntimeOrigin::signed(HOST_LEGAL_OFFICER), LOC_ID));
        let vote_id: u64 = LogionVote::last_vote_id();
        assert_ok!(LogionVote::vote(RuntimeOrigin::signed(HOST_LEGAL_OFFICER), vote_id, true));
        assert_ok!(LogionVote::vote(RuntimeOrigin::signed(LEGAL_OFFICER2), vote_id, false));
        assert_eq!(LogionVote::votes(vote_id), Some(
            Vote {
                loc_id: LOC_ID,
                ballots: vec![
                    Ballot { voter: HOST_LEGAL_OFFICER, status: BallotStatus::VotedYes },
                    Ballot { voter: LEGAL_OFFICER2, status: BallotStatus::VotedNo },
                ]
            }));
        assert_eq!(LogionVote::is_vote_closed_and_approved(vote_id), (true, false))
    });
}

#[test]
fn it_votes_yes_and_yes() {
    new_test_ext().execute_with(|| {
        assert_empty_storage();
        assert_ok!(LogionVote::create_vote_for_all_legal_officers(RuntimeOrigin::signed(HOST_LEGAL_OFFICER), LOC_ID));
        let vote_id: u64 = LogionVote::last_vote_id();
        assert_ok!(LogionVote::vote(RuntimeOrigin::signed(HOST_LEGAL_OFFICER), vote_id, true));
        assert_ok!(LogionVote::vote(RuntimeOrigin::signed(LEGAL_OFFICER2), vote_id, true));
        assert_eq!(LogionVote::votes(vote_id), Some(
            Vote {
                loc_id: LOC_ID,
                ballots: vec![
                    Ballot { voter: HOST_LEGAL_OFFICER, status: BallotStatus::VotedYes },
                    Ballot { voter: LEGAL_OFFICER2, status: BallotStatus::VotedYes },
                ]
            }));
        assert_eq!(LogionVote::is_vote_closed_and_approved(vote_id), (true, true))
    });
}

#[test]
fn it_fails_to_vote_wrong_vote_id() {
    new_test_ext().execute_with(|| {
        assert_empty_storage();
        assert_ok!(LogionVote::create_vote_for_all_legal_officers(RuntimeOrigin::signed(HOST_LEGAL_OFFICER), LOC_ID));
        let wrong_vote_id: u64 = LogionVote::last_vote_id() + 100;
        assert_err!(LogionVote::vote(RuntimeOrigin::signed(LEGAL_OFFICER2), wrong_vote_id, false), Error::<Test>::VoteNotFound);
    });
}

#[test]
fn it_fails_to_vote_wrong_voter() {
    new_test_ext().execute_with(|| {
        assert_empty_storage();
        assert_ok!(LogionVote::create_vote_for_all_legal_officers(RuntimeOrigin::signed(HOST_LEGAL_OFFICER), LOC_ID));
        let vote_id: u64 = LogionVote::last_vote_id();
        assert_err!(LogionVote::vote(RuntimeOrigin::signed(WALLET_USER), vote_id, true), Error::<Test>::NotAllowed);
    });
}

#[test]
fn it_fails_to_vote_twice() {
    new_test_ext().execute_with(|| {
        assert_empty_storage();
        assert_ok!(LogionVote::create_vote_for_all_legal_officers(RuntimeOrigin::signed(HOST_LEGAL_OFFICER), LOC_ID));
        let vote_id: u64 = LogionVote::last_vote_id();
        assert_ok!(LogionVote::vote(RuntimeOrigin::signed(LEGAL_OFFICER2), vote_id, false));
        assert_err!(LogionVote::vote(RuntimeOrigin::signed(LEGAL_OFFICER2), vote_id, true), Error::<Test>::AlreadyVoted);
    });
}

fn assert_empty_storage() {
    assert_eq!(LogionVote::votes(0), None);
    assert_eq!(LogionVote::votes(1), None);
    assert_eq!(LogionVote::last_vote_id(), 0);
}
