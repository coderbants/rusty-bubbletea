use rusty_bubbletea::key::{
    Key, KeyMod, KeyMsg, KeyPressMsg, KeyReleaseMsg, KEY_BACKSPACE, KEY_DOWN, KEY_END, KEY_ENTER,
    KEY_ESCAPE, KEY_HOME, KEY_LEFT, KEY_PG_DOWN, KEY_PG_UP, KEY_RIGHT, KEY_SPACE, KEY_TAB, KEY_UP,
};

#[test]
fn test_v2_key_press_msg() {
    let k = Key::new('a', "a", KeyMod::default());
    let msg = KeyPressMsg(k.clone());
    assert_eq!(msg.to_string(), "a");
    assert_eq!(format!("{}", k), "a");

    let space_key = Key::new(' ', " ", KeyMod::default());
    let space_msg = KeyPressMsg(space_key);
    assert_eq!(space_msg.to_string(), "space");

    let rel_msg = KeyReleaseMsg(k);
    assert_eq!(format!("{}", rel_msg), "a");

    let press_enum = KeyMsg::Press(space_msg.clone());
    assert_eq!(format!("{}", press_enum), "space");
    assert_eq!(press_enum.key().string(), "space");

    let rel_enum = KeyMsg::Release(rel_msg);
    assert_eq!(format!("{}", rel_enum), "a");
}

#[test]
fn test_key_modifiers_and_special_symbols() {
    let mut mods = KeyMod::default();
    assert!(!mods.contains(KeyMod::CTRL));
    mods = KeyMod(KeyMod::CTRL.0 | KeyMod::ALT.0 | KeyMod::SHIFT.0 | KeyMod::META.0);
    assert!(mods.contains(KeyMod::CTRL));
    assert!(mods.contains(KeyMod::ALT));
    assert!(mods.contains(KeyMod::SHIFT));
    assert!(mods.contains(KeyMod::META));

    let key_ctrl_c = Key::new('c', "", KeyMod::CTRL);
    assert_eq!(key_ctrl_c.keystroke(), "ctrl+c");

    let key_combo = Key::new('x', "", mods);
    assert_eq!(key_combo.keystroke(), "ctrl+alt+shift+meta+x");

    let specials = [
        (KEY_UP, "up"),
        (KEY_DOWN, "down"),
        (KEY_RIGHT, "right"),
        (KEY_LEFT, "left"),
        (KEY_HOME, "home"),
        (KEY_END, "end"),
        (KEY_PG_UP, "pgup"),
        (KEY_PG_DOWN, "pgdown"),
        (KEY_ENTER, "enter"),
        (KEY_TAB, "tab"),
        (KEY_BACKSPACE, "backspace"),
        (KEY_ESCAPE, "esc"),
        (KEY_SPACE, "space"),
    ];

    for (code, name) in specials {
        let k = Key::new(code, "", KeyMod::default());
        assert_eq!(k.keystroke(), name);
    }
}
