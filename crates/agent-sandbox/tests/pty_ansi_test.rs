use agent_sandbox::pty::ansi::strip_ansi;

fn stripped(input: &[u8]) -> String {
    String::from_utf8_lossy(&strip_ansi(input)).into_owned()
}

#[test]
fn passes_plain_text_through_unchanged() {
    assert_eq!(stripped(b"hello\nworld\r\n\ttabbed"), "hello\nworld\r\n\ttabbed");
}

#[test]
fn strips_csi_sequences() {
    assert_eq!(stripped(b"\x1b[31mred\x1b[0m plain"), "red plain");
}

#[test]
fn strips_csi_with_multiple_parameters() {
    assert_eq!(stripped(b"\x1b[1;37;40mstyled\x1b[m"), "styled");
}

#[test]
fn strips_osc_terminated_by_bel() {
    assert_eq!(stripped(b"\x1b]0;window title\x07visible"), "visible");
}

#[test]
fn strips_osc_terminated_by_string_terminator() {
    assert_eq!(stripped(b"\x1b]0;window title\x1b\\visible"), "visible");
}

#[test]
fn strips_dcs_sequences() {
    assert_eq!(stripped(b"\x1bPsome dcs payload\x1b\\visible"), "visible");
}

#[test]
fn drops_bare_c0_controls_except_newline_tab_and_carriage_return() {
    // BEL (0x07) and backspace (0x08) dropped; \n \r \t preserved.
    assert_eq!(stripped(b"a\x07b\x08c\nd\re\tf"), "abc\nd\re\tf");
}

#[test]
fn truncated_csi_at_buffer_end_does_not_panic_and_drops_the_partial_sequence() {
    assert_eq!(stripped(b"before\x1b[31"), "before");
}

#[test]
fn truncated_osc_at_buffer_end_does_not_panic() {
    assert_eq!(stripped(b"before\x1b]0;unterminated title"), "before");
}

#[test]
fn lone_escape_at_buffer_end_does_not_panic() {
    assert_eq!(stripped(b"before\x1b"), "before");
}

#[test]
fn empty_input_returns_empty_output() {
    assert_eq!(strip_ansi(b""), Vec::<u8>::new());
}
