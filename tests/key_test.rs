use charming_bubbletea::*;

#[test]
fn test_key_msg() {
    let key = KeyMsg::new(KeyType::KeyEnter);
    assert_eq!(key.to_string_rep(), "enter");
    assert_eq!(format!("{}", key), "enter");

    let runes = vec!['a'];
    let key_a = KeyMsg::from_runes(&runes, false);
    assert_eq!(key_a.to_string_rep(), "a");

    let key_alt_a = KeyMsg::from_runes(&runes, true);
    assert_eq!(key_alt_a.to_string_rep(), "alt+a");

    let ctrl_c = KeyMsg::new(KeyType::KeyCtrlC);
    assert_eq!(ctrl_c.to_string_rep(), "ctrl+c");
}
