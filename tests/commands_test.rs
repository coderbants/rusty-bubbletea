use rusty_bubbletea::{batch, quit, sequence};

#[test]
fn test_commands() {
    let q = quit();
    assert!(q.is_some());

    let b = batch(vec![quit()]);
    assert!(b.is_some());

    let s = sequence(vec![quit()]);
    assert!(s.is_some());
}
