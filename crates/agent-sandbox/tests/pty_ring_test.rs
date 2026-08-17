use agent_sandbox::pty::ring::ScrollbackRing;

#[test]
fn push_within_capacity_round_trips() {
    let mut ring = ScrollbackRing::with_capacity(16);
    ring.push(b"hello");
    assert_eq!(ring.snapshot(), b"hello");
    assert_eq!(ring.dropped_bytes(), 0);
}

#[test]
fn wraparound_keeps_the_newest_bytes() {
    let mut ring = ScrollbackRing::with_capacity(8);
    ring.push(b"ABCDEFGH");
    ring.push(b"XYZ");
    // Last 8 bytes of the 11-byte stream "ABCDEFGHXYZ".
    assert_eq!(ring.snapshot(), b"DEFGHXYZ");
    assert_eq!(ring.dropped_bytes(), 3);
}

#[test]
fn a_single_push_larger_than_capacity_keeps_the_tail() {
    let mut ring = ScrollbackRing::with_capacity(4);
    ring.push(b"ABCDEFGHIJ");
    assert_eq!(ring.snapshot(), b"GHIJ");
    assert_eq!(ring.dropped_bytes(), 6);
}

#[test]
fn tail_lines_returns_the_last_n_lines_preserving_trailing_newline() {
    let mut ring = ScrollbackRing::with_capacity(256);
    ring.push(b"line1\nline2\nline3\n");
    assert_eq!(ring.tail_lines(2), b"line2\nline3\n");
    assert_eq!(ring.tail_lines(1), b"line3\n");
}

#[test]
fn tail_lines_without_trailing_newline_does_not_add_one() {
    let mut ring = ScrollbackRing::with_capacity(256);
    ring.push(b"a\nb\nc");
    assert_eq!(ring.tail_lines(2), b"b\nc");
}

#[test]
fn tail_lines_more_than_available_returns_everything() {
    let mut ring = ScrollbackRing::with_capacity(256);
    ring.push(b"only\nthree\nlines\n");
    assert_eq!(ring.tail_lines(100), b"only\nthree\nlines\n");
}

#[test]
fn tail_lines_zero_is_empty() {
    let mut ring = ScrollbackRing::with_capacity(256);
    ring.push(b"a\nb\n");
    assert!(ring.tail_lines(0).is_empty());
}

#[test]
fn tail_lines_survives_the_wrap_boundary_splitting_a_line() {
    // Capacity chosen so a later push wraps mid-line; correctness here is
    // "does not panic and returns something UTF-8-lossy-decodable", since
    // callers are expected to decode with `from_utf8_lossy`.
    let mut ring = ScrollbackRing::with_capacity(10);
    ring.push(b"aaaa\nbbbb\n");
    ring.push(b"cc\n");
    let out = ring.tail_lines(2);
    assert!(!out.is_empty());
    let _ = String::from_utf8_lossy(&out);
}
