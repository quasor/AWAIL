fn main() {
    // Ensure plugin resource paths exist so tauri_build doesn't fail during
    // `cargo check` when plugins haven't been built yet. Real builds always
    // run `cargo xtask build-plugin` first which produces the actual files.
    let bundled = std::path::Path::new("../../target/bundled");
    for name in ["wail-plugin-send", "wail-plugin-recv"] {
        let clap = bundled.join(format!("{name}.clap"));
        if !clap.exists() {
            std::fs::create_dir_all(bundled).ok();
            std::fs::write(&clap, []).ok();
        }
        let vst3 = bundled.join(format!("{name}.vst3"));
        if !vst3.exists() {
            std::fs::create_dir_all(&vst3).ok();
        }
    }

    // When CI caches the target/ directory, stale tauri_build output in OUT_DIR
    // can conflict with fresh resource processing (file/directory mismatches).
    // Clean it before each build to ensure a fresh start.
    if let Ok(out_dir) = std::env::var("OUT_DIR") {
        let _ = std::fs::remove_dir_all(&out_dir);
        let _ = std::fs::create_dir_all(&out_dir);
    }

    tauri_build::build();
}
