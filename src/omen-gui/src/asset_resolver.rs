use std::path::Path;

pub fn get_asset_path(filename: &str) -> String {
    let system_path = format!("/usr/share/omen-space/assets/{}", filename);
    if Path::new(&system_path).exists() {
        return system_path;
    }
    
    // Fallback for local development
    format!("assets/{}", filename)
}
