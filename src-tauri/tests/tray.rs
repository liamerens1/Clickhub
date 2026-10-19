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
fn pause_label_reflects_state() {
    assert_eq!(MenuAction::pause_label(false), "Pause");
    assert_eq!(MenuAction::pause_label(true), "Resume");
    assert_eq!(MenuAction::Pause.label(), "Pause");
}

#[test]
fn only_the_debug_aids_are_debug_items() {
    let debug: Vec<MenuAction> = MenuAction::ALL
        .into_iter()
        .filter(|action| action.is_debug())
        .collect();
    assert_eq!(
        debug,
        vec![MenuAction::SimulateHookLoss, MenuAction::StallCore]
    );
}
