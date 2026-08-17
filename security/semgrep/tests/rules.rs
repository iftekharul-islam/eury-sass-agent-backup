// Rust semgrep rule test fixtures

fn bad_std_fs() {
    // ruleid: eury-no-std-fs-outside-sandbox
    let _ = std::fs::read_to_string("foo.txt");
}

fn ok_sandbox_call() {
    // ok: eury-no-std-fs-outside-sandbox
    let _ = agent_sandbox::read_file("foo.txt");
}

fn bad_command_new() {
    // ruleid: eury-no-command-new-outside-sandbox
    let _ = std::process::Command::new("sh");
}

fn ok_sandbox_command() {
    // ok: eury-no-command-new-outside-sandbox
    let _ = agent_sandbox::run_command("cargo test");
}

fn bad_unwrap(opt: Option<i32>) {
    // ruleid: eury-no-unwrap-in-core-paths
    let _ = opt.unwrap();

    // ruleid: eury-no-unwrap-in-core-paths
    let _ = opt.expect("failed");

    // ruleid: eury-no-unwrap-in-core-paths
    panic!("unrecoverable error");
}

fn ok_error_handling(opt: Option<i32>) -> Result<i32, &'static str> {
    // ok: eury-no-unwrap-in-core-paths
    match opt {
        Some(v) => Ok(v),
        None => Err("missing value"),
    }
}
