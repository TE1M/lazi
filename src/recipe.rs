use serde::Deserialize;

// Structs for YAML parsing.
// Option wrapper states a field can be missing without crashing the program.

#[derive(Debug, Deserialize)]
pub struct LaziRecipe {
    pub name: String,
    pub vm: VmConfig,
    pub user: Option<UserConfig>,
    pub packages: Option<PackagesConfig>,
    pub files: Option<Vec<FileConfig>>,
    pub scripts: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct VmConfig {
    pub cpus: u8,
    pub ram_mb: u32,
}

#[derive(Debug, Deserialize)]
pub struct UserConfig {
    pub username: String,
    pub shell: String,
}

#[derive(Debug, Deserialize)]
pub struct PackagesConfig {
    pub apt: Option<Vec<String>>,
    pub pipx: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct FileConfig {
    pub path: String,
    pub content: Option<String>,
}
