use crate::{mock::*, LegalOfficerData, Error, HostData, LegalOfficerDataOf, HostDataOf};
use frame_support::{assert_err, assert_ok, error::BadOrigin};
use logion_shared::IsLegalOfficer;
use sp_core::OpaquePeerId;

const LEGAL_OFFICER_ID: u64 = 1;
const ANOTHER_ID: u64 = 2;
const LEGAL_OFFICER_ID2: u64 = 3;
const LEGAL_OFFICER_ID3: u64 = 4;

impl Default for LegalOfficerDataOf<Test> {
    fn default() -> Self {
        LegalOfficerData::Host(Default::default())
    }
}

impl TryFrom<LegalOfficerDataOf<Test>> for HostDataOf<Test> {
    type Error = ();

    fn try_from(value: LegalOfficerDataOf<Test>) -> Result<Self, Self::Error> {
        match value {
            LegalOfficerData::Host(data) => Ok(data),
            _ => Err(())
        }
    }
}

#[test]
fn it_adds_host() {
    new_test_ext().execute_with(|| {
        assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, Default::default()));
        assert!(LoAuthorityList::legal_officer_set(LEGAL_OFFICER_ID).is_some());
        assert!(LoAuthorityList::legal_officer_nodes().is_empty());
    });
}

#[test]
fn it_removes_host() {
    new_test_ext().execute_with(|| {
        assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, Default::default()));
        assert_ok!(LoAuthorityList::remove_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID));
        assert!(LoAuthorityList::legal_officer_set(LEGAL_OFFICER_ID).is_none());
        assert!(LoAuthorityList::legal_officer_nodes().is_empty());
    });
}

#[test]
fn it_fails_adding_if_not_superuser() {
    new_test_ext().execute_with(|| {
        assert_err!(LoAuthorityList::add_legal_officer(RuntimeOrigin::signed(0), LEGAL_OFFICER_ID, Default::default()), BadOrigin);
    });
}

#[test]
fn it_fails_removing_if_not_superuser() {
    new_test_ext().execute_with(|| {
        assert_err!(LoAuthorityList::remove_legal_officer(RuntimeOrigin::signed(0), LEGAL_OFFICER_ID), BadOrigin);
    });
}

#[test]
fn it_ensures_origin_ok_as_expected() {
    new_test_ext().execute_with(|| {
        assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, Default::default()));
        assert_ok!(LoAuthorityList::ensure_legal_officer(RuntimeOrigin::signed(LEGAL_OFFICER_ID)));
    });
}

#[test]
fn it_ensures_origin_err_as_expected() {
    new_test_ext().execute_with(|| {
        assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, Default::default()));
        let result = LoAuthorityList::ensure_legal_officer(RuntimeOrigin::signed(ANOTHER_ID));
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
fn it_gets_legal_officers() {
    new_test_ext().execute_with(|| {
        assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, Default::default()));
        assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID2, Default::default()));
        assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID3, Default::default()));
        let legal_officers = LoAuthorityList::legal_officers();
        assert_eq!(legal_officers.len(), 3);
        assert!(legal_officers.contains(&LEGAL_OFFICER_ID));
        assert!(legal_officers.contains(&LEGAL_OFFICER_ID2));
        assert!(legal_officers.contains(&LEGAL_OFFICER_ID3));
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
fn it_lets_host_update() {
    new_test_ext().execute_with(|| {
        assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, Default::default()));
        let base_url = "https://node.logion.network".as_bytes().to_vec();
        let node_id = OpaquePeerId(bs58::decode("12D3KooWBmAwcd4PJNJvfV89HwE48nwkRmAgo8Vy3uQEyNNHBox2").into_vec().unwrap());
        assert_ok!(LoAuthorityList::update_legal_officer(RuntimeOrigin::signed(LEGAL_OFFICER_ID), LEGAL_OFFICER_ID, LegalOfficerData::Host(HostData {
            node_id: Option::Some(node_id.clone()),
            base_url: Option::Some(base_url.clone()),
            region: Region::Europe,
        })));
        let data: HostDataOf<Test> = LoAuthorityList::legal_officer_set(LEGAL_OFFICER_ID).unwrap().try_into().unwrap();
        assert_eq!(data.base_url.unwrap(), base_url);
        assert_eq!(data.node_id.unwrap(), node_id);
        assert_eq!(LoAuthorityList::legal_officer_nodes().len(), 1);
        assert!(LoAuthorityList::legal_officer_nodes().contains(&node_id));
    });
}

#[test]
fn it_lets_superuser_update() {
    new_test_ext().execute_with(|| {
        assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, Default::default()));
        let base_url = "https://node.logion.network".as_bytes().to_vec();
        let node_id = OpaquePeerId(bs58::decode("12D3KooWBmAwcd4PJNJvfV89HwE48nwkRmAgo8Vy3uQEyNNHBox2").into_vec().unwrap());
        assert_ok!(LoAuthorityList::update_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, LegalOfficerData::Host(HostData {
            node_id: Option::Some(node_id.clone()),
            base_url: Option::Some(base_url.clone()),
            region: Region::Europe,
        })));
        let data: HostDataOf<Test> = LoAuthorityList::legal_officer_set(LEGAL_OFFICER_ID).unwrap().try_into().unwrap();
        assert_eq!(data.base_url.unwrap(), base_url);
        assert_eq!(data.node_id.unwrap(), node_id);
        assert_eq!(LoAuthorityList::legal_officer_nodes().len(), 1);
        assert!(LoAuthorityList::legal_officer_nodes().contains(&node_id));
    });
}

#[test]
fn it_fails_add_if_peer_id_already_in_use() {
    new_test_ext().execute_with(|| {
        let base_url1 = "https://node1.logion.network".as_bytes().to_vec();
        let node_id1 = OpaquePeerId(bs58::decode("12D3KooWBmAwcd4PJNJvfV89HwE48nwkRmAgo8Vy3uQEyNNHBox2").into_vec().unwrap());
        assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, LegalOfficerData::Host(HostData {
            node_id: Option::Some(node_id1.clone()),
            base_url: Option::Some(base_url1.clone()),
            region: Region::Europe,
        })));

        let base_url2 = "https://node2.logion.network".as_bytes().to_vec();
        assert_err!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID2, LegalOfficerData::Host(HostData {
            base_url: Option::Some(base_url2.clone()),
            node_id: Option::Some(node_id1.clone()),
            region: Region::Europe,
        })), Error::<Test>::PeerIdAlreadyInUse);
    });
}

#[test]
fn it_fails_update_if_peer_id_already_in_use() {
    new_test_ext().execute_with(|| {
        let base_url1 = "https://node1.logion.network".as_bytes().to_vec();
        let node_id1 = OpaquePeerId(bs58::decode("12D3KooWBmAwcd4PJNJvfV89HwE48nwkRmAgo8Vy3uQEyNNHBox2").into_vec().unwrap());
        assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, LegalOfficerData::Host(HostData {
            node_id: Option::Some(node_id1.clone()),
            base_url: Option::Some(base_url1.clone()),
            region: Region::Europe,
        })));

        let base_url2 = "https://node2.logion.network".as_bytes().to_vec();
        let node_id2 = OpaquePeerId(bs58::decode("12D3KooWQYV9dGMFoRzNStwpXztXaBUjtPqi6aU76ZgUriHhKust").into_vec().unwrap());
        assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID2, LegalOfficerData::Host(HostData {
            base_url: Option::Some(base_url2.clone()),
            node_id: Option::Some(node_id2.clone()),
            region: Region::Europe,
        })));
        assert_eq!(LoAuthorityList::legal_officer_nodes().len(), 2);
        assert!(LoAuthorityList::legal_officer_nodes().contains(&node_id1));
        assert!(LoAuthorityList::legal_officer_nodes().contains(&node_id2));

        assert_err!(LoAuthorityList::update_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID2, LegalOfficerData::Host(HostData {
            base_url: Option::Some(base_url2.clone()),
            node_id: Option::Some(node_id1.clone()),
            region: Region::Europe,
        })), Error::<Test>::PeerIdAlreadyInUse);
    });
}

#[test]
fn it_updates_nodes_on_remove() {
    new_test_ext().execute_with(|| {
        let base_url = "https://node.logion.network".as_bytes().to_vec();
        let node_id = OpaquePeerId(bs58::decode("12D3KooWBmAwcd4PJNJvfV89HwE48nwkRmAgo8Vy3uQEyNNHBox2").into_vec().unwrap());
        assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, LegalOfficerData::Host(HostData {
            node_id: Option::Some(node_id.clone()),
            base_url: Option::Some(base_url.clone()),
            region: Region::Europe,
        })));
        assert_ok!(LoAuthorityList::remove_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID));
        assert!(LoAuthorityList::legal_officer_nodes().is_empty());
    });
}

#[test]
fn it_updates_nodes_on_update() {
    new_test_ext().execute_with(|| {
        let base_url = "https://node.logion.network".as_bytes().to_vec();
        let node_id1 = OpaquePeerId(bs58::decode("12D3KooWBmAwcd4PJNJvfV89HwE48nwkRmAgo8Vy3uQEyNNHBox2").into_vec().unwrap());
        assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, LegalOfficerData::Host(HostData {
            node_id: Option::Some(node_id1.clone()),
            base_url: Option::Some(base_url.clone()),
            region: Region::Europe,
        })));
        let node_id2 = OpaquePeerId(bs58::decode("12D3KooWQYV9dGMFoRzNStwpXztXaBUjtPqi6aU76ZgUriHhKust").into_vec().unwrap());
        assert_ok!(LoAuthorityList::update_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, LegalOfficerData::Host(HostData {
            node_id: Option::Some(node_id2.clone()),
            base_url: Option::Some(base_url.clone()),
            region: Region::Europe,
        })));

        assert_eq!(LoAuthorityList::legal_officer_nodes().len(), 1);
        assert!(LoAuthorityList::legal_officer_nodes().contains(&node_id2));
    });
}

#[test]
fn it_adds_guest() {
    new_test_ext().execute_with(|| {
        setup_host_and_guest();
        assert!(LoAuthorityList::legal_officer_set(LEGAL_OFFICER_ID2).is_some());
        assert_eq!(LoAuthorityList::legal_officer_nodes().len(), 1);
    });
}

fn setup_host_and_guest() {
    let host_data = LegalOfficerData::Host(HostData {
        node_id: Option::Some(OpaquePeerId(bs58::decode("12D3KooWBmAwcd4PJNJvfV89HwE48nwkRmAgo8Vy3uQEyNNHBox2").into_vec().unwrap())),
        base_url: Option::Some("https://node.logion.network".as_bytes().to_vec()),
        region: Region::Europe,
    });
    assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, host_data));
    assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID2, LegalOfficerData::Guest(LEGAL_OFFICER_ID)));
}

#[test]
fn it_removes_guest() {
    new_test_ext().execute_with(|| {
        setup_host_and_guest();
        assert_ok!(LoAuthorityList::remove_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID2));
        assert!(LoAuthorityList::legal_officer_set(LEGAL_OFFICER_ID2).is_none());
        assert_eq!(LoAuthorityList::legal_officer_nodes().len(), 1);
    });
}

#[test]
fn it_turns_guest_into_host() {
    new_test_ext().execute_with(|| {
        setup_host_and_guest();
        let host_data2 = LegalOfficerData::Host(HostData {
            node_id: Option::Some(OpaquePeerId(bs58::decode("12D3KooWQYV9dGMFoRzNStwpXztXaBUjtPqi6aU76ZgUriHhKust").into_vec().unwrap())),
            base_url: Option::Some("https://node2.logion.network".as_bytes().to_vec()),
            region: Region::Europe,
        });
        assert_ok!(LoAuthorityList::update_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID2, host_data2));
        assert!(LoAuthorityList::legal_officer_set(LEGAL_OFFICER_ID2).is_some());
        assert_eq!(LoAuthorityList::legal_officer_nodes().len(), 2);
    });
}

#[test]
fn it_turns_host_into_guest() {
    new_test_ext().execute_with(|| {
        setup_hosts();
        let host_data = LegalOfficerData::Host(HostData {
            node_id: Option::Some(OpaquePeerId(bs58::decode("12D3KooWJvyP3VJYymTqG7eH4PM5rN4T2agk5cdNCfNymAqwqcvZ").into_vec().unwrap())),
            base_url: Option::Some("https://node3.logion.network".as_bytes().to_vec()),
            region: Region::Other,
        });
        assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID3, host_data));
        assert_eq!(LoAuthorityList::legal_officer_nodes().len(), 3);
        assert_ok!(LoAuthorityList::update_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID2, LegalOfficerData::Guest(LEGAL_OFFICER_ID3)));
        assert!(LoAuthorityList::legal_officer_set(LEGAL_OFFICER_ID2).is_some());
        assert_eq!(LoAuthorityList::legal_officer_nodes().len(), 2);
    });
}

fn setup_hosts() {
    let host_data = LegalOfficerData::Host(HostData {
        node_id: Option::Some(OpaquePeerId(bs58::decode("12D3KooWBmAwcd4PJNJvfV89HwE48nwkRmAgo8Vy3uQEyNNHBox2").into_vec().unwrap())),
        base_url: Option::Some("https://node1.logion.network".as_bytes().to_vec()),
        region: Region::Europe,
    });
    assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, host_data));
    let host_data2 = LegalOfficerData::Host(HostData {
        node_id: Option::Some(OpaquePeerId(bs58::decode("12D3KooWQYV9dGMFoRzNStwpXztXaBUjtPqi6aU76ZgUriHhKust").into_vec().unwrap())),
        base_url: Option::Some("https://node2.logion.network".as_bytes().to_vec()),
        region: Region::Other,
    });
    assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID2, host_data2));
}

#[test]
fn it_fails_turning_host_with_guest_into_guest() {
    new_test_ext().execute_with(|| {
        setup_host_and_guest();
        let host_data3 = LegalOfficerData::Host(HostData {
            node_id: Option::Some(OpaquePeerId(bs58::decode("12D3KooWJvyP3VJYymTqG7eH4PM5rN4T2agk5cdNCfNymAqwqcvZ").into_vec().unwrap())),
            base_url: Option::Some("https://node3.logion.network".as_bytes().to_vec()),
            region: Region::Europe,
        });
        assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID3, host_data3));
        assert_err!(LoAuthorityList::update_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, LegalOfficerData::Guest(LEGAL_OFFICER_ID3)),
            Error::<Test>::HostHasGuest);
    });
}

#[test]
fn it_fails_removing_host_with_guests() {
    new_test_ext().execute_with(|| {
        setup_host_and_guest();
        assert_err!(LoAuthorityList::remove_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID), Error::<Test>::HostHasGuest);
    });
}

#[test]
fn it_fails_adding_guest_with_guest() {
    new_test_ext().execute_with(|| {
        setup_host_and_guest();
        assert_err!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID3, LegalOfficerData::Guest(LEGAL_OFFICER_ID2)),
            Error::<Test>::GuestOfGuest);
    });
}

#[test]
fn it_fails_adding_guest_with_unknown_host() {
    new_test_ext().execute_with(|| {
        let host_data = LegalOfficerData::Host(HostData {
            node_id: Option::Some(OpaquePeerId(bs58::decode("12D3KooWBmAwcd4PJNJvfV89HwE48nwkRmAgo8Vy3uQEyNNHBox2").into_vec().unwrap())),
            base_url: Option::Some("https://node.logion.network".as_bytes().to_vec()),
            region: Region::Europe,
        });
        assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID, host_data));
        assert_err!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID2, LegalOfficerData::Guest(LEGAL_OFFICER_ID3)),
            Error::<Test>::HostNotFound);
    });
}

#[test]
fn it_fails_if_guest_updates() {
    new_test_ext().execute_with(|| {
        setup_host_and_guest();
        assert_err!(LoAuthorityList::update_legal_officer(RuntimeOrigin::signed(LEGAL_OFFICER_ID2), LEGAL_OFFICER_ID2, LegalOfficerData::Guest(LEGAL_OFFICER_ID)),
            Error::<Test>::GuestCannotUpdate);
    });
}

#[test]
fn it_fails_if_host_converts_to_guest() {
    new_test_ext().execute_with(|| {
        setup_hosts();
        assert_err!(LoAuthorityList::update_legal_officer(RuntimeOrigin::signed(LEGAL_OFFICER_ID), LEGAL_OFFICER_ID, LegalOfficerData::Guest(LEGAL_OFFICER_ID2)),
            Error::<Test>::HostCannotConvert);
    });
}

#[test]
fn it_fails_changing_host_host_region() {
    new_test_ext().execute_with(|| {
        setup_hosts();
        assert_err!(LoAuthorityList::update_legal_officer(RuntimeOrigin::signed(LEGAL_OFFICER_ID), LEGAL_OFFICER_ID, LegalOfficerData::Host(HostData {
            node_id: Option::Some(OpaquePeerId(bs58::decode("12D3KooWBmAwcd4PJNJvfV89HwE48nwkRmAgo8Vy3uQEyNNHBox2").into_vec().unwrap())),
            base_url: Option::Some("https://node1.logion.network".as_bytes().to_vec()),
            region: Region::Other,
        })), Error::<Test>::CannotChangeRegion);
    });
}

#[test]
fn it_fails_changing_guest_host_region() {
    new_test_ext().execute_with(|| {
        setup_host_and_guest();
        let host_data = LegalOfficerData::Host(HostData {
            node_id: Option::Some(OpaquePeerId(bs58::decode("12D3KooWQYV9dGMFoRzNStwpXztXaBUjtPqi6aU76ZgUriHhKust").into_vec().unwrap())),
            base_url: Option::Some("https://node2.logion.network".as_bytes().to_vec()),
            region: Region::Other,
        });
        assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID3, host_data));
        assert_err!(LoAuthorityList::update_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID2, LegalOfficerData::Guest(LEGAL_OFFICER_ID3)), Error::<Test>::CannotChangeRegion);
    });
}

#[test]
fn it_fails_changing_host_guest_region() {
    new_test_ext().execute_with(|| {
        setup_hosts();
        assert_err!(LoAuthorityList::update_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID2, LegalOfficerData::Guest(LEGAL_OFFICER_ID)), Error::<Test>::CannotChangeRegion);
    });
}

#[test]
fn it_fails_changing_guest_guest_region() {
    new_test_ext().execute_with(|| {
        setup_hosts();
        assert_ok!(LoAuthorityList::add_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID3, LegalOfficerData::Guest(LEGAL_OFFICER_ID)));
        assert_err!(LoAuthorityList::update_legal_officer(RuntimeOrigin::root(), LEGAL_OFFICER_ID3, LegalOfficerData::Guest(LEGAL_OFFICER_ID2)), Error::<Test>::CannotChangeRegion);
    });
}
