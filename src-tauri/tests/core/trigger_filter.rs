use clickhub_lib::config::{KeyboardTrigger, Modifier, MouseButton, MouseTrigger, Trigger};
use clickhub_lib::core::{Button, ModifierSet, TriggerFilter};

fn mouse(button: MouseButton, modifiers: Vec<Modifier>, swallow: bool) -> Trigger {
    Trigger::Mouse(MouseTrigger {
        button,
        modifiers,
        swallow,
    })
}

#[test]
fn matches_the_trigger_button_with_no_modifiers() {
    let filter = TriggerFilter::from_trigger(&mouse(MouseButton::X2, vec![], false));
    assert!(filter.matches(Button::X2, ModifierSet::EMPTY));
    assert!(!filter.matches(Button::X1, ModifierSet::EMPTY));
    assert!(!filter.matches(Button::Left, ModifierSet::EMPTY));
}

#[test]
fn requires_exactly_the_configured_modifiers() {
    let filter = TriggerFilter::from_trigger(&mouse(MouseButton::X2, vec![Modifier::Ctrl], true));
    let ctrl: ModifierSet = [Modifier::Ctrl].into_iter().collect();
    let ctrl_shift: ModifierSet = [Modifier::Ctrl, Modifier::Shift].into_iter().collect();

    assert!(filter.matches(Button::X2, ctrl));
    assert!(!filter.matches(Button::X2, ModifierSet::EMPTY));
    assert!(!filter.matches(Button::X2, ctrl_shift));
    assert!(filter.swallow);
}

#[test]
fn plain_trigger_does_not_fire_while_a_modifier_is_held() {
    let filter = TriggerFilter::from_trigger(&mouse(MouseButton::Middle, vec![], false));
    let shift: ModifierSet = [Modifier::Shift].into_iter().collect();
    assert!(!filter.matches(Button::Middle, shift));
}

#[test]
fn keyboard_trigger_disables_mouse_matching() {
    let filter = TriggerFilter::from_trigger(&Trigger::Keyboard(KeyboardTrigger {
        shortcut: "Ctrl+Alt+Space".into(),
    }));
    assert_eq!(filter, TriggerFilter::DISABLED);
    for button in [
        Button::Left,
        Button::Right,
        Button::Middle,
        Button::X1,
        Button::X2,
    ] {
        assert!(!filter.matches(button, ModifierSet::EMPTY));
    }
}
