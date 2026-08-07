use charming_bubbletea::*;

#[test]
fn test_commands() {
    let q_cmd = quit();
    assert!(q_cmd.is_some());
    let msg = (q_cmd.unwrap())().unwrap();
    assert!(msg.as_ref().as_any().is::<QuitMsg>());

    let alt_cmd = enter_alt_screen();
    let msg = (alt_cmd.unwrap())().unwrap();
    assert!(msg.as_ref().as_any().is::<EnterAltScreenMsg>());

    let exit_alt_cmd = exit_alt_screen();
    let msg = (exit_alt_cmd.unwrap())().unwrap();
    assert!(msg.as_ref().as_any().is::<ExitAltScreenMsg>());

    let b_cmd = batch(vec![quit(), enter_alt_screen()]);
    assert!(b_cmd.is_some());
    let msg = (b_cmd.unwrap())().unwrap();
    assert!(msg.as_ref().as_any().is::<BatchMsg>());

    let empty_b_cmd = batch(vec![]);
    assert!(empty_b_cmd.is_none());
}

#[test]
fn test_window_size() {
    let ws = WindowSizeMsg::new(80, 24);
    assert_eq!(ws.width, 80);
    assert_eq!(ws.height, 24);
}
