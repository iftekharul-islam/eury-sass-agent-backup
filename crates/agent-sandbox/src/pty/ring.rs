/// Fixed-capacity scrollback buffer for one PTY session.
///
/// Backs `terminal_capture`: every byte the PTY produces is pushed here and
/// the newest data is never dropped, independent of whatever the display
/// coalescer (`coalesce.rs`) chooses to drop for rendering. A single
/// contiguous `Box<[u8]>` with wraparound is used instead of
/// `VecDeque<Vec<u8>>` so a sustained 10 MB/s stream does not churn a heap
/// allocation per chunk.
pub struct ScrollbackRing {
    buf: Box<[u8]>,
    cap: usize,
    /// Index one past the most recently written byte.
    head: usize,
    /// Number of valid bytes currently held, `<= cap`.
    len: usize,
    dropped_bytes: u64,
}

impl ScrollbackRing {
    /// # Panics
    ///
    /// Panics if `cap` is zero.
    #[must_use]
    pub fn with_capacity(cap: usize) -> Self {
        assert!(cap > 0, "ScrollbackRing capacity must be non-zero");
        Self { buf: vec![0u8; cap].into_boxed_slice(), cap, head: 0, len: 0, dropped_bytes: 0 }
    }

    pub fn push(&mut self, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }

        if bytes.len() >= self.cap {
            let start = bytes.len() - self.cap;
            self.buf.copy_from_slice(&bytes[start..]);
            self.dropped_bytes += start as u64 + self.len as u64;
            self.head = 0;
            self.len = self.cap;
            return;
        }

        let n = bytes.len();
        let first = (self.cap - self.head).min(n);
        self.buf[self.head..self.head + first].copy_from_slice(&bytes[..first]);
        if first < n {
            let second = n - first;
            self.buf[..second].copy_from_slice(&bytes[first..]);
        }
        self.head = (self.head + n) % self.cap;

        let new_len = self.len + n;
        if new_len > self.cap {
            self.dropped_bytes += (new_len - self.cap) as u64;
            self.len = self.cap;
        } else {
            self.len = new_len;
        }
    }

    /// Bytes currently held, oldest first.
    #[must_use]
    pub fn snapshot(&self) -> Vec<u8> {
        let start = (self.head + self.cap - self.len) % self.cap;
        let mut out = Vec::with_capacity(self.len);
        if start + self.len <= self.cap {
            out.extend_from_slice(&self.buf[start..start + self.len]);
        } else {
            let first = self.cap - start;
            out.extend_from_slice(&self.buf[start..]);
            out.extend_from_slice(&self.buf[..self.len - first]);
        }
        out
    }

    /// The last `n` newline-delimited lines, oldest first. A trailing
    /// newline in the source is preserved but does not itself count as an
    /// extra empty line. Tolerant of the ring boundary splitting a
    /// multi-byte UTF-8 sequence — callers must decode lossily.
    #[must_use]
    pub fn tail_lines(&self, n: usize) -> Vec<u8> {
        if n == 0 {
            return Vec::new();
        }
        let data = self.snapshot();
        let had_trailing_newline = data.last() == Some(&b'\n');
        let mut lines: Vec<&[u8]> = data.split(|&b| b == b'\n').collect();
        if had_trailing_newline {
            lines.pop();
        }
        let start = lines.len().saturating_sub(n);
        let mut out = lines[start..].join(&b'\n');
        if had_trailing_newline {
            out.push(b'\n');
        }
        out
    }

    #[must_use]
    pub fn dropped_bytes(&self) -> u64 {
        self.dropped_bytes
    }

    #[must_use]
    pub fn capacity(&self) -> usize {
        self.cap
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}
