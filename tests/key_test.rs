use rusty_bubbletea::{Key, KeyMod, KeyPressMsg};

#[test]
fn test_v2_key_press_msg() {
    let k = Key::new('a', "a", KeyMod::default());
    let msg = KeyPressMsg(k);
    assert_eq!(msg.to_string(), "a");

    let space_key = Key::new(' ', " ", KeyMod::default());
    let space_msg = KeyPressMsg(space_key);
    assert_eq!(space_msg.to_string(), "space");
}
