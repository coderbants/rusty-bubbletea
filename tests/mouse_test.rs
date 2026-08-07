use charming_bubbletea::*;

#[test]
fn test_mouse_msg() {
    let mouse = MouseMsg::new(10, 20, MouseButton::MouseLeft, MouseAction::MouseActionPress);
    assert_eq!(mouse.x, 10);
    assert_eq!(mouse.y, 20);
    assert_eq!(mouse.button, MouseButton::MouseLeft);
    assert_eq!(mouse.action, MouseAction::MouseActionPress);

    let display_str = format!("{}", mouse);
    assert!(display_str.contains("10, 20"));
}
