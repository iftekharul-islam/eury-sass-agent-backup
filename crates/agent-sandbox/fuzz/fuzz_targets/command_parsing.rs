#![no_main]
use libfuzzer_sys::fuzz_target;
use agent_sandbox::command::CommandGuard;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = CommandGuard::parse_and_verify(s);
    }
});
