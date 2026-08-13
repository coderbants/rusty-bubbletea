use rusty_bubbletea::{KeyMod, Mouse, MouseButton, MouseClickMsg};

#[test]
fn test_v2_mouse_msg() {
    let m = Mouse {
        x: 10,
        y: 20,
        button: MouseButton::MouseLeft,
        mod_keys: KeyMod::default(),
    };
    let msg = MouseClickMsg(m);
    assert_eq!(msg.0.x, 10);
    assert_eq!(msg.0.y, 20);
    assert_eq!(msg.0.button, MouseButton::MouseLeft);
}
