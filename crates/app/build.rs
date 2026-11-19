fn main() {
    // Time of the build (seconds since 1970); the user interface shows it in local time.
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    println!("cargo:rustc-env=PP_BUILD_EPOCH={secs}");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=../ui");
    println!("cargo:rerun-if-changed=tauri.conf.json");
    tauri_build::build()
}
