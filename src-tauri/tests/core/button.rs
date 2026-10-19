use clickhub_lib::config::MouseButton;
use clickhub_lib::core::Button;

const ALL: [Button; 5] = [
    Button::Left,
    Button::Right,
    Button::Middle,
    Button::X1,
    Button::X2,
];

#[test]
fn codes_round_trip_and_zero_means_none() {
    for button in ALL {
        assert_ne!(button.code(), 0);
        assert_eq!(Button::from_code(button.code()), Some(button));
    }
    assert_eq!(Button::from_code(0), None);
    assert_eq!(Button::from_code(6), None);
}

#[test]
fn trigger_buttons_map_to_hook_buttons() {
    assert_eq!(Button::from(MouseButton::Right), Button::Right);
    assert_eq!(Button::from(MouseButton::Middle), Button::Middle);
    assert_eq!(Button::from(MouseButton::X1), Button::X1);
    assert_eq!(Button::from(MouseButton::X2), Button::X2);
}
