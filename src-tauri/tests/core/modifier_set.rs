use clickhub_lib::config::Modifier;
use clickhub_lib::core::ModifierSet;

#[test]
fn collects_modifiers_into_distinct_bits() {
    let set: ModifierSet = [Modifier::Ctrl, Modifier::Win].into_iter().collect();
    assert!(set.contains(Modifier::Ctrl));
    assert!(set.contains(Modifier::Win));
    assert!(!set.contains(Modifier::Shift));
    assert!(!set.contains(Modifier::Alt));
}

#[test]
fn bits_round_trip_and_ignore_unknown_bits() {
    let all: ModifierSet = [
        Modifier::Ctrl,
        Modifier::Shift,
        Modifier::Alt,
        Modifier::Win,
    ]
    .into_iter()
    .collect();
    assert_eq!(ModifierSet::from_bits(all.bits()), all);
    assert_eq!(ModifierSet::from_bits(0xF0), ModifierSet::EMPTY);
}
