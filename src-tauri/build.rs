fn main() {
    tauri_build::build();
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_manifest_file("manifest.xml");
        res.compile().unwrap();
    }
}
