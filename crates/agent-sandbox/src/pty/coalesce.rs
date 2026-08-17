/// One coalesced batch of PTY output, ready to send over the wire.
#[derive(Debug, Clone)]
pub struct OutputFrame {
    pub seq: u64,
    pub bytes: Vec<u8>,
    /// Bytes dropped from the *head* of this batch because it exceeded
    /// `max_frame_bytes` — a terminal viewport shows the tail of the
    /// stream, so on overflow we keep the newest bytes and drop the oldest
    /// ones accumulated since the last flush.
    pub dropped_before: u64,
}

/// Accumulates PTY bytes between flushes and hands back at most one bounded
/// frame per flush.
///
/// This exists so coalescing happens in Rust, before bytes reach the IPC
/// boundary — coalescing only on the frontend still pays full per-read IPC
/// cost for a stream that can produce thousands of small reads per second,
/// and cannot hit the 10 MB/s throughput target. The caller (the per-session
/// task in `manager.rs`) is expected to call `append` from the PTY reader
/// thread and `flush` on a fixed interval (16 ms, matching a 60 Hz frame).
pub struct FrameCoalescer {
    pending: Vec<u8>,
    max_frame_bytes: usize,
    seq: u64,
}

impl FrameCoalescer {
    #[must_use]
    pub fn new(max_frame_bytes: usize) -> Self {
        Self { pending: Vec::new(), max_frame_bytes, seq: 0 }
    }

    pub fn append(&mut self, bytes: &[u8]) {
        self.pending.extend_from_slice(bytes);
    }

    /// Returns `None` when there is nothing pending, so an idle session
    /// produces zero IPC traffic instead of empty heartbeat frames.
    pub fn flush(&mut self) -> Option<OutputFrame> {
        if self.pending.is_empty() {
            return None;
        }

        let dropped_before = if self.pending.len() > self.max_frame_bytes {
            let excess = self.pending.len() - self.max_frame_bytes;
            self.pending.drain(..excess);
            excess as u64
        } else {
            0
        };

        self.seq += 1;
        Some(OutputFrame {
            seq: self.seq,
            bytes: std::mem::take(&mut self.pending),
            dropped_before,
        })
    }
}
