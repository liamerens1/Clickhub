//! Mirrors `src/config/shortcut.rs`. The same cases are tested against the TypeScript parser in
//! `tests/shared/schema/shortcut.test.ts`.

use clickhub_lib::config::{Key, Modifier, NamedKey, Shortcut};

fn parse(text: &str) -> Shortcut {
    Shortcut::parse(text).unwrap_or_else(|error| panic!("{text} should parse: {error}"))
}

#[test]
fn parses_modifiers_and_keys() {
    let shortcut = parse("Ctrl+Shift+Esc");
    assert_eq!(shortcut.modifiers, vec![Modifier::Ctrl, Modifier::Shift]);
    assert_eq!(shortcut.key, Key::Named(NamedKey::Esc));

    assert_eq!(parse("a").key, Key::Letter('A'));
    assert_eq!(parse("7").key, Key::Digit(7));
    assert_eq!(parse("F24").key, Key::Function(24));
    assert_eq!(
        parse("MediaPlayPause").key,
        Key::Named(NamedKey::MediaPlayPause)
    );
}

#[test]
fn is_case_insensitive_and_ignores_spaces() {
    assert_eq!(parse(" win + shift + s ").to_string(), "Win+Shift+S");
    assert_eq!(
        parse("CONTROL+windows+pageup").to_string(),
        "Ctrl+Win+PageUp"
    );
}

#[test]
fn accepts_aliases() {
    assert_eq!(parse("Escape").to_string(), "Esc");
    assert_eq!(parse("Return").to_string(), "Enter");
    assert_eq!(parse("Ctrl+Alt+Del").to_string(), "Ctrl+Alt+Delete");
}

#[test]
fn rejects_bad_shortcuts() {
    for bad in [
        "",
        "+",
        "Ctrl+",
        "Ctrl",
        "Ctrl+Ctrl+A",
        "Hyper+A",
        "F0",
        "F25",
        "F01",
        "Ctrl+AB",
        "Shift+Ctrl",
        "A+B",
    ] {
        assert!(Shortcut::parse(bad).is_err(), "`{bad}` should be rejected");
    }
}
