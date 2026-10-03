use petunia_design_text::{TextEditBuffer, TextSelection};
#[test]
fn deletion_and_navigation_respect_combining_zwj_and_flag_graphemes() {
    let original = "a\u{301}👩‍💻🇧🇷";
    let mut buffer = TextEditBuffer::new(original.into()).unwrap();
    buffer.move_to(original.len(), false);
    for expected in ["a\u{301}👩‍💻", "a\u{301}", ""] {
        assert!(buffer.delete(true).unwrap());
        assert_eq!(buffer.content(), expected);
    }
    for _ in 0..3 {
        assert!(buffer.undo());
    }
    assert_eq!(buffer.content(), original);
    buffer.set_selection(1, 4);
    assert_eq!(
        buffer.selection(),
        TextSelection {
            anchor: 0,
            focus: 3
        }
    );
    buffer.move_horizontal(true, false);
    assert_eq!(buffer.selection().focus, 3);
    buffer.move_horizontal(true, true);
    assert_eq!(buffer.selection().range(), 3..14);
}
#[test]
fn ime_composition_is_derived_until_one_atomic_commit_and_can_cancel() {
    let mut buffer = TextEditBuffer::new("Olá mundo".into()).unwrap();
    buffer.set_selection(5, 10);
    buffer.set_preedit("日本語".into(), Some((1, 6))).unwrap();
    assert_eq!(buffer.content(), "Olá mundo");
    assert_eq!(buffer.display_content(), "Olá 日本語");
    assert_eq!(
        buffer.display_selection(),
        TextSelection {
            anchor: 5,
            focus: 11
        }
    );
    buffer.cancel_preedit();
    assert_eq!(buffer.display_content(), buffer.content());
    buffer.set_preedit("日本語".into(), None).unwrap();
    assert!(buffer.insert("日本語").unwrap());
    assert!(!buffer.has_preedit());
    assert_eq!(buffer.content(), "Olá 日本語");
    assert!(buffer.undo());
    assert_eq!(buffer.content(), "Olá mundo");
    assert!(buffer.redo());
    assert_eq!(buffer.content(), "Olá 日本語");
}
#[test]
fn admission_failure_keeps_content_selection_composition_and_history() {
    let mut buffer = TextEditBuffer::new("a".repeat(65536)).unwrap();
    buffer.move_to(65536, false);
    let previous = buffer.clone();
    assert!(buffer.insert("x").is_err());
    assert_eq!(buffer, previous);
    assert!(buffer.set_preedit("x".into(), None).is_err());
    assert_eq!(buffer, previous);
    assert!(!buffer.undo());
    buffer.select_all();
    buffer.insert("é").unwrap();
    assert!(buffer.undo());
    assert_eq!(buffer.content().len(), 65536);
}
#[test]
fn combining_insert_joins_graphemes_without_leaving_an_interior_caret() {
    let mut buffer = TextEditBuffer::new("ab".into()).unwrap();
    buffer.move_to(1, false);
    buffer.insert("\u{301}").unwrap();
    assert_eq!(buffer.content(), "a\u{301}b");
    assert_eq!(buffer.selection().focus, 3);
    buffer.delete(true).unwrap();
    assert_eq!(buffer.content(), "b");
}
