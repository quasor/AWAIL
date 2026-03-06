use std::path::PathBuf;
use std::process::Command;

/// On macOS, a valid plugin bundle is a directory (e.g. foo.clap/Contents/MacOS/…).
/// A 0-byte file or missing path is not valid.
/// On Linux/Windows, a valid bundle is a non-empty file.
fn bundle_is_valid(path: &PathBuf) -> bool {
    #[cfg(target_os = "macos")]
    return path.is_dir();
    #[cfg(not(target_os = "macos"))]
    return path.is_file() && path.metadata().map(|m| m.len() > 0).unwrap_or(false);
}

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let workspace_root = PathBuf::from(&manifest_dir)
        .parent()
        .unwrap() // crates/wail-plugin-test -> crates/
        .parent()
        .unwrap() // crates/ -> workspace root
        .to_path_buf();

    let recv_bundle = workspace_root.join("target/bundled/wail-plugin-recv.clap");
    let send_bundle = workspace_root.join("target/bundled/wail-plugin-send.clap");

    if !bundle_is_valid(&recv_bundle) || !bundle_is_valid(&send_bundle) {
        println!("cargo:warning=Plugin bundles missing — running `cargo xtask bundle-plugin --debug`");
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
        let status = Command::new(&cargo)
            .args(["xtask", "bundle-plugin", "--debug"])
            .current_dir(&workspace_root)
            .status()
            .expect("Failed to spawn cargo xtask bundle-plugin");
        assert!(status.success(), "cargo xtask bundle-plugin --debug failed");
    }

    // Rebuild if the plugin bundles are replaced
    println!("cargo:rerun-if-changed={}", recv_bundle.display());
    println!("cargo:rerun-if-changed={}", send_bundle.display());
    // Rebuild if plugin source changes
    println!(
        "cargo:rerun-if-changed={}",
        workspace_root.join("crates/wail-plugin-recv/src").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        workspace_root.join("crates/wail-plugin-send/src").display()
    );
}
