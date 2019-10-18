use std::path::PathBuf;

pub struct Resources {
    root_dir: PathBuf,
}

impl Resources {
    pub fn new(root_dir: PathBuf) -> Result<Resources, String> {
        if !root_dir.exists() {
            return Err(format!(
                "Directory '{}' has not found",
                root_dir.to_string_lossy()
            ));
        }
        Ok(Resources { root_dir })
    }

    pub fn get_font_path(&self, file_name: &str) -> Result<PathBuf, String> {
        self.get_path("fonts", file_name)
    }

    pub fn get_sprite_path(&self, file_name: &str) -> Result<PathBuf, String> {
        self.get_path("sprites", file_name)
    }

    fn get_path(&self, resource_type: &str, file_name: &str) -> Result<PathBuf, String> {
        let path = self.root_dir.join(resource_type).join(file_name);
        if !path.exists() {
            return Err(format!(
                "File {} has not found in {} resources",
                file_name, resource_type
            ));
        }
        Ok(path)
    }
}
