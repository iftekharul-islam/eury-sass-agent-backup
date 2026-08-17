fn main() {
    tauri_build::build();

    #[cfg(target_os = "linux")]
    build_linux_probe();
}

#[cfg(target_os = "linux")]
fn build_linux_probe() {
    use std::path::PathBuf;
    use std::process::Command;

    if std::env::var_os("AGENT_SANDBOX_PROBE_BUILD_IN_PROGRESS").is_some() {
        return;
    }

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.join("../../..");
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let target_subdir = if profile == "dev" || profile == "debug" {
        "debug"
    } else {
        profile.as_str()
    };
    let target_dir = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| workspace_root.join("target"));
    let probe_src = target_dir.join(target_subdir).join("agent-sandbox-probe");

    println!(
        "cargo:rerun-if-changed={}",
        workspace_root.join("crates/agent-sandbox-probe/src").display()
    );

    if !probe_src.is_file() {
        let mut cmd = Command::new("cargo");
        cmd.env("AGENT_SANDBOX_PROBE_BUILD_IN_PROGRESS", "1")
            .args(["build", "-p", "agent-sandbox-probe"])
            .current_dir(&workspace_root);
        if let Ok(target_dir) = std::env::var("CARGO_TARGET_DIR") {
            cmd.env("CARGO_TARGET_DIR", target_dir);
        }
        let status = cmd
            .status()
            .expect("failed to spawn cargo for agent-sandbox-probe");
        if !status.success() {
            panic!("failed to build agent-sandbox-probe");
        }
    }

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let probe_dst = out_dir.join("agent-sandbox-probe");
    std::fs::copy(&probe_src, &probe_dst).unwrap_or_else(|err| {
        panic!(
            "failed to copy {} to {}: {err}",
            probe_src.display(),
            probe_dst.display()
        );
    });
    println!("cargo:rustc-env=AGENT_SANDBOX_PROBE={}", probe_dst.display());
}
