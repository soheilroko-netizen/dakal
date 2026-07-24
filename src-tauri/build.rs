fn main() {
    tauri_build::build();
    embed_resource::compile("manifest.rc", embed_resource::NONE);
}
