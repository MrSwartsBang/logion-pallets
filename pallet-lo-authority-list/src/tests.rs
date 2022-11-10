use crate::{mock::*, LegalOfficerData, Error};
use frame_support::{assert_err, assert_ok, error::BadOrigin, traits::EnsureOrigin};
use logion_shared::IsLegalOfficer;
use sp_core::OpaquePeerId;

const LEGAL_OFFICER_ID: u64 = 1;
const ANOTHER_ID: u64 = 2;
const LEGAL_OFFICER_ID2: u64 = 3;

#[test]
fn it_adds_lo() {
	new_test_ext().execute_with(|| {
		assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, Default::default()));
		assert!(LoAuthorityList::legal_officer_set(LEGAL_OFFICER_ID).is_some());
		assert!(LoAuthorityList::legal_officer_nodes().is_empty());
	});
}

#[test]
fn it_removes_lo() {
	new_test_ext().execute_with(|| {
		assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, Default::default()));
		assert_ok!(LoAuthorityList::remove_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID));
		assert!(LoAuthorityList::legal_officer_set(LEGAL_OFFICER_ID).is_none());
		assert!(LoAuthorityList::legal_officer_nodes().is_empty());
	});
}

#[test]
fn it_fails_adding_if_not_manager() {
	new_test_ext().execute_with(|| {
		assert_err!(LoAuthorityList::add_legal_officer(RuntimeOrigin::signed(0), LEGAL_OFFICER_ID, Default::default()), BadOrigin);
	});
}

#[test]
fn it_fails_removing_if_not_manager() {
	new_test_ext().execute_with(|| {
		assert_err!(LoAuthorityList::remove_legal_officer(RuntimeOrigin::signed(0), LEGAL_OFFICER_ID), BadOrigin);
	});
}

#[test]
fn it_ensures_origin_ok_as_expected() {
	new_test_ext().execute_with(|| {
		assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, Default::default()));
		assert_ok!(LoAuthorityList::try_origin(RuntimeOrigin::signed(LEGAL_OFFICER_ID)));
	});
}

#[test]
fn it_ensures_origin_err_as_expected() {
	new_test_ext().execute_with(|| {
		assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, Default::default()));
		let result = LoAuthorityList::try_origin(RuntimeOrigin::signed(ANOTHER_ID));
		assert!(result.err().is_some());
	});
}

#[test]
fn it_detects_legal_officer() {
	new_test_ext().execute_with(|| {
		assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, Default::default()));
		assert!(LoAuthorityList::is_legal_officer(&LEGAL_OFFICER_ID));
	});
}

#[test]
fn it_detects_regular_user() {
	new_test_ext().execute_with(|| {
		assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, Default::default()));
		assert!(!LoAuthorityList::is_legal_officer(&ANOTHER_ID));
	});
}

#[test]
fn it_lets_lo_update() {
	new_test_ext().execute_with(|| {
		assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, Default::default()));
		let base_url = "https://node.logion.network".as_bytes().to_vec();
		let node_id = OpaquePeerId(bs58::decode("12D3KooWBmAwcd4PJNJvfV89HwE48nwkRmAgo8Vy3uQEyNNHBox2").into_vec().unwrap());
		assert_ok!(LoAuthorityList::update_legal_officer(RuntimeOrigin::signed(LEGAL_OFFICER_ID), LEGAL_OFFICER_ID, LegalOfficerData {
			node_id: Option::Some(node_id.clone()),
			base_url: Option::Some(base_url.clone()),
		}));
		assert_eq!(LoAuthorityList::legal_officer_set(LEGAL_OFFICER_ID).unwrap().base_url.unwrap(), base_url);
		assert_eq!(LoAuthorityList::legal_officer_set(LEGAL_OFFICER_ID).unwrap().node_id.unwrap(), node_id);
		assert_eq!(LoAuthorityList::legal_officer_nodes().len(), 1);
		assert!(LoAuthorityList::legal_officer_nodes().contains(&node_id));
	});
}

#[test]
fn it_lets_manager_update() {
	new_test_ext().execute_with(|| {
		assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, Default::default()));
		let base_url = "https://node.logion.network".as_bytes().to_vec();
		let node_id = OpaquePeerId(bs58::decode("12D3KooWBmAwcd4PJNJvfV89HwE48nwkRmAgo8Vy3uQEyNNHBox2").into_vec().unwrap());
		assert_ok!(LoAuthorityList::update_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, LegalOfficerData {
			node_id: Option::Some(node_id.clone()),
			base_url: Option::Some(base_url.clone()),
		}));
		assert_eq!(LoAuthorityList::legal_officer_set(LEGAL_OFFICER_ID).unwrap().base_url.unwrap(), base_url);
		assert_eq!(LoAuthorityList::legal_officer_set(LEGAL_OFFICER_ID).unwrap().node_id.unwrap(), node_id);
		assert_eq!(LoAuthorityList::legal_officer_nodes().len(), 1);
		assert!(LoAuthorityList::legal_officer_nodes().contains(&node_id));
	});
}

#[test]
fn it_fails_add_if_peer_id_already_in_use() {
	new_test_ext().execute_with(|| {
		let base_url1 = "https://node1.logion.network".as_bytes().to_vec();
		let node_id1 = OpaquePeerId(bs58::decode("12D3KooWBmAwcd4PJNJvfV89HwE48nwkRmAgo8Vy3uQEyNNHBox2").into_vec().unwrap());
		assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, LegalOfficerData {
			node_id: Option::Some(node_id1.clone()),
			base_url: Option::Some(base_url1.clone()),
		}));

		let base_url2 = "https://node2.logion.network".as_bytes().to_vec();
		assert_err!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID2, LegalOfficerData {
			base_url: Option::Some(base_url2.clone()),
			node_id: Option::Some(node_id1.clone()),
		}), Error::<Test>::PeerIdAlreadyInUse);
	});
}

#[test]
fn it_fails_update_if_peer_id_already_in_use() {
	new_test_ext().execute_with(|| {
		let base_url1 = "https://node1.logion.network".as_bytes().to_vec();
		let node_id1 = OpaquePeerId(bs58::decode("12D3KooWBmAwcd4PJNJvfV89HwE48nwkRmAgo8Vy3uQEyNNHBox2").into_vec().unwrap());
		assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, LegalOfficerData {
			node_id: Option::Some(node_id1.clone()),
			base_url: Option::Some(base_url1.clone()),
		}));

		let base_url2 = "https://node2.logion.network".as_bytes().to_vec();
		let node_id2 = OpaquePeerId(bs58::decode("12D3KooWQYV9dGMFoRzNStwpXztXaBUjtPqi6aU76ZgUriHhKust").into_vec().unwrap());
		assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID2, LegalOfficerData {
			base_url: Option::Some(base_url2.clone()),
			node_id: Option::Some(node_id2.clone()),
		}));
		assert_eq!(LoAuthorityList::legal_officer_nodes().len(), 2);
		assert!(LoAuthorityList::legal_officer_nodes().contains(&node_id1));
		assert!(LoAuthorityList::legal_officer_nodes().contains(&node_id2));

		assert_err!(LoAuthorityList::update_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID2, LegalOfficerData {
			base_url: Option::Some(base_url2.clone()),
			node_id: Option::Some(node_id1.clone()),
		}), Error::<Test>::PeerIdAlreadyInUse);
	});
}

#[test]
fn it_updates_nodes_on_remove() {
	new_test_ext().execute_with(|| {
		let base_url = "https://node.logion.network".as_bytes().to_vec();
		let node_id = OpaquePeerId(bs58::decode("12D3KooWBmAwcd4PJNJvfV89HwE48nwkRmAgo8Vy3uQEyNNHBox2").into_vec().unwrap());
		assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, LegalOfficerData {
			node_id: Option::Some(node_id.clone()),
			base_url: Option::Some(base_url.clone()),
		}));
		assert_ok!(LoAuthorityList::remove_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID));
		assert!(LoAuthorityList::legal_officer_nodes().is_empty());
	});
}

#[test]
fn it_updates_nodes_on_update() {
	new_test_ext().execute_with(|| {
		let base_url = "https://node.logion.network".as_bytes().to_vec();
		let node_id1 = OpaquePeerId(bs58::decode("12D3KooWBmAwcd4PJNJvfV89HwE48nwkRmAgo8Vy3uQEyNNHBox2").into_vec().unwrap());
		assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, LegalOfficerData {
			node_id: Option::Some(node_id1.clone()),
			base_url: Option::Some(base_url.clone()),
		}));
		let node_id2 = OpaquePeerId(bs58::decode("12D3KooWQYV9dGMFoRzNStwpXztXaBUjtPqi6aU76ZgUriHhKust").into_vec().unwrap());
		assert_ok!(LoAuthorityList::update_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, LegalOfficerData {
			node_id: Option::Some(node_id2.clone()),
			base_url: Option::Some(base_url.clone()),
		}));

		assert_eq!(LoAuthorityList::legal_officer_nodes().len(), 1);
		assert!(LoAuthorityList::legal_officer_nodes().contains(&node_id2));
	});
}
