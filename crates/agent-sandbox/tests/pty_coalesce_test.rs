use agent_sandbox::pty::coalesce::FrameCoalescer;

#[test]
fn flush_is_none_when_idle() {
    let mut coalescer = FrameCoalescer::new(1024);
    assert!(coalescer.flush().is_none());
}

#[test]
fn flush_returns_pending_bytes_with_incrementing_seq() -> Result<(), Box<dyn std::error::Error>> {
    let mut coalescer = FrameCoalescer::new(1024);

    coalescer.append(b"hello ");
    coalescer.append(b"world");
    let Some(frame) = coalescer.flush() else {
        return Err("expected a frame after appending bytes".into());
    };
    assert_eq!(frame.seq, 1);
    assert_eq!(frame.bytes, b"hello world");
    assert_eq!(frame.dropped_before, 0);

    assert!(coalescer.flush().is_none(), "second flush with nothing new pending should be None");

    coalescer.append(b"again");
    let Some(frame2) = coalescer.flush() else {
        return Err("expected a second frame".into());
    };
    assert_eq!(frame2.seq, 2);
    assert_eq!(frame2.bytes, b"again");
    Ok(())
}

#[test]
fn overflow_drops_the_oldest_bytes_and_keeps_the_tail() -> Result<(), Box<dyn std::error::Error>> {
    let mut coalescer = FrameCoalescer::new(4);
    coalescer.append(b"ABCDEFGH"); // 8 bytes appended between flushes, cap is 4
    let Some(frame) = coalescer.flush() else {
        return Err("expected a frame".into());
    };
    assert_eq!(frame.bytes, b"EFGH");
    assert_eq!(frame.dropped_before, 4);
    Ok(())
}

#[test]
fn idle_session_produces_zero_frames_across_many_flushes() {
    let mut coalescer = FrameCoalescer::new(1024);
    for _ in 0..50 {
        assert!(coalescer.flush().is_none());
    }
}
