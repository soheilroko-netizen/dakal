fn main() {
    tauri_build::build();
    #[cfg(target_os = "windows")]
    embed_resource::compile("manifest.xml", embed_resource::NONE);
}
