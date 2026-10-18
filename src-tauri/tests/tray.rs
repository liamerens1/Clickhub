//! Mirrors `src/tray/`.

use std::collections::HashSet;

use clickhub_lib::tray::MenuAction;

#[test]
fn menu_ids_are_unique() {
    let ids: HashSet<&str> = MenuAction::ALL.iter().map(|action| action.id()).collect();
    assert_eq!(ids.len(), MenuAction::ALL.len());
}

#[test]
fn every_menu_id_maps_back_to_its_action() {
    for action in MenuAction::ALL {
        assert_eq!(MenuAction::from_id(action.id()), Some(action));
    }
    assert_eq!(MenuAction::from_id("not_a_menu_item"), None);
}

#[test]
fn pause_is_disabled_until_phase_1() {
    assert!(!MenuAction::Pause.enabled());
    assert!(MenuAction::OpenSettings.enabled());
    assert!(MenuAction::Quit.enabled());
}
