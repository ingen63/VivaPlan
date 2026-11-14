fn main() {
    // Zeitpunkt des Builds (Sekunden seit 1970); die Oberfläche zeigt ihn in der Ortszeit an.
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
