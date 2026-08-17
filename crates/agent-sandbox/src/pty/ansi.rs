/// Strips `ANSI` escape sequences (CSI, OSC, DCS, and other C1 string
/// sequences) and non-printable C0 controls from PTY output, for
/// `terminal_capture` and terminal-selection sharing.
///
/// A hand-written state machine rather than a regex: CSI/OSC/DCS terminator
/// rules (parameter/intermediate byte ranges, BEL-or-ST termination) are
/// exactly the kind of thing a regex gets subtly wrong, especially at a
/// truncated buffer boundary. Truncated sequences at the end of `input` are
/// handled gracefully — whatever was accumulated before the truncation is
/// returned, nothing panics.
#[must_use]
pub fn strip_ansi(input: &[u8]) -> Vec<u8> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum State {
        Normal,
        Escape,
        Csi,
        /// OSC / DCS / PM / APC: terminated by BEL (0x07) or ST (ESC \).
        StringTerminated,
    }

    let mut out = Vec::with_capacity(input.len());
    let mut state = State::Normal;
    let mut i = 0;

    while i < input.len() {
        let b = input[i];
        match state {
            State::Normal => {
                if b == 0x1B {
                    state = State::Escape;
                } else if b == b'\n' || b == b'\r' || b == b'\t' || (b >= 0x20 && b != 0x7F) {
                    out.push(b);
                }
                // else: drop other C0 controls (BEL, backspace, ...) and DEL.
            }
            State::Escape => {
                state = match b {
                    b'[' => State::Csi,
                    b']' | b'P' | b'X' | b'^' | b'_' => State::StringTerminated,
                    _ => State::Normal, // two-byte escape, e.g. ESC c, ESC =, ESC (
                };
            }
            State::Csi => {
                // Parameter/intermediate bytes are 0x20..=0x3F; the final
                // byte is 0x40..=0x7E. Anything else ends the sequence too,
                // so a malformed CSI can't wedge the parser in this state.
                if (0x40..=0x7E).contains(&b) {
                    state = State::Normal;
                }
            }
            State::StringTerminated => {
                if b == 0x07 {
                    state = State::Normal;
                } else if b == 0x1B && input.get(i + 1) == Some(&b'\\') {
                    i += 1; // consume the trailing backslash of ST too
                    state = State::Normal;
                }
            }
        }
        i += 1;
    }

    out
}
