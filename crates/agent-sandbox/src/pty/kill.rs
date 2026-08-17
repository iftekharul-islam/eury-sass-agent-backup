//! Process-group signal primitives for terminating a PTY session.
//!
//! Deliberately not part of `crate::process::ProcessSupervisor`:
//! `ProcessSupervisor::spawn_and_wait` consumes its child via
//! `wait_with_output()`, which is precisely why its own timeout branch
//! cannot kill anything (see its comments). A PTY session needs kill-by-id
//! from another task at an arbitrary point in an unbounded lifetime, plus a
//! two-stage SIGHUP-then-SIGKILL escalation — a different shape, not a
//! missing feature of the same one. `ProcessSupervisor`'s own missing
//! process-group handling is pre-existing debt, tracked separately.

#[cfg(unix)]
mod imp {
    use nix::sys::signal::{Signal, killpg};
    use nix::unistd::{Pid, getpgid};

    /// Safe wrappers over `killpg`/`getpgid` — no `unsafe`, so
    /// `unsafe_code = "forbid"` holds without a crate-level exception.
    #[derive(Debug, Clone, Copy)]
    pub struct PtyKiller {
        pgid: Option<Pid>,
    }

    impl PtyKiller {
        /// Reads the child's actual process group rather than assuming
        /// `pgid == pid` — `portable-pty` makes the child a session leader
        /// on unix, which usually implies this, but it is read, not assumed.
        #[must_use]
        pub fn capture(pid: Option<u32>) -> Self {
            let resolved = pid.and_then(|p| getpgid(Some(Pid::from_raw(p.cast_signed()))).ok());
            Self { pgid: resolved }
        }

        pub fn hangup(&self) {
            if let Some(pgid) = self.pgid {
                let _ = killpg(pgid, Signal::SIGHUP);
            }
        }

        pub fn kill_hard(&self) {
            if let Some(pgid) = self.pgid {
                let _ = killpg(pgid, Signal::SIGKILL);
            }
        }

        /// Signal 0: existence check, no signal actually delivered.
        #[must_use]
        pub fn is_alive(&self) -> bool {
            match self.pgid {
                Some(pgid) => killpg(pgid, None::<Signal>).is_ok(),
                None => false,
            }
        }
    }
}

#[cfg(not(unix))]
mod imp {
    /// Windows termination goes through `Child::kill()` plus dropping the
    /// `ConPTY` master (closing the pseudoconsole terminates attached
    /// processes) — see `PtySession::terminate`. This type exists so
    /// `session.rs` doesn't need platform `cfg`s of its own.
    #[derive(Debug, Clone, Copy)]
    pub struct PtyKiller;

    impl PtyKiller {
        #[must_use]
        pub fn capture(_pid: Option<u32>) -> Self {
            Self
        }
        pub fn hangup(&self) {}
        pub fn kill_hard(&self) {}
        #[must_use]
        pub fn is_alive(&self) -> bool {
            false
        }
    }
}

pub use imp::PtyKiller;
