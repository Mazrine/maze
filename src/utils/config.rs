use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use uuid::Uuid;

use crate::modules::{AudioModule, Connection};

/// Main project configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub version: String,
    pub sample_rate: f64,
    pub buffer_size: usize,
    pub modules: HashMap<Uuid, AudioModule>,
    pub connections: Vec<Connection>,
    pub metadata: ProjectMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub created_at: String,
    pub modified_at: String,
    pub author: String,
    pub description: String,
    pub tags: Vec<String>,
}

impl Default for ProjectMetadata {
    fn default() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            created_at: now.clone(),
            modified_at: now,
            author: "Unknown".to_string(),
            description: "".to_string(),
            tags: Vec::new(),
        }
    }
}

impl ProjectConfig {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            sample_rate: 44100.0,
            buffer_size: 256,
            modules: HashMap::new(),
            connections: Vec::new(),
            metadata: ProjectMetadata::default(),
        }
    }

    /// Save project to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Load project from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: ProjectConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Update modification timestamp
    pub fn touch(&mut self) {
        self.metadata.modified_at = chrono::Utc::now().to_rfc3339();
    }

    /// Add a module to the project
    pub fn add_module(&mut self, module: AudioModule) {
        self.modules.insert(module.id, module);
        self.touch();
    }

    /// Remove a module and its connections
    pub fn remove_module(&mut self, module_id: Uuid) {
        self.modules.remove(&module_id);
        self.connections
            .retain(|conn| conn.from_module != module_id && conn.to_module != module_id);
        self.touch();
    }

    /// Add a connection
    pub fn add_connection(&mut self, connection: Connection) -> anyhow::Result<()> {
        // Validate that modules exist
        if !self.modules.contains_key(&connection.from_module) {
            return Err(anyhow::anyhow!("Source module not found"));
        }
        if !self.modules.contains_key(&connection.to_module) {
            return Err(anyhow::anyhow!("Destination module not found"));
        }

        // Check for duplicate connections
        if self.connections.iter().any(|conn| {
            conn.from_module == connection.from_module
                && conn.from_output == connection.from_output
                && conn.to_module == connection.to_module
                && conn.to_input == connection.to_input
        }) {
            return Err(anyhow::anyhow!("Connection already exists"));
        }

        self.connections.push(connection);
        self.touch();
        Ok(())
    }

    /// Remove a connection
    pub fn remove_connection(
        &mut self,
        from_module: Uuid,
        from_output: usize,
        to_module: Uuid,
        to_input: usize,
    ) {
        self.connections.retain(|conn| {
            !(conn.from_module == from_module
                && conn.from_output == from_output
                && conn.to_module == to_module
                && conn.to_input == to_input)
        });
        self.touch();
    }

    /// Get connections for a specific module
    pub fn get_module_connections(&self, module_id: Uuid) -> (Vec<&Connection>, Vec<&Connection>) {
        let inputs: Vec<&Connection> = self
            .connections
            .iter()
            .filter(|conn| conn.to_module == module_id)
            .collect();

        let outputs: Vec<&Connection> = self
            .connections
            .iter()
            .filter(|conn| conn.from_module == module_id)
            .collect();

        (inputs, outputs)
    }
}

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub audio: AudioSettings,
    pub ui: UISettings,
    pub keybindings: KeyBindings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    pub sample_rate: f64,
    pub buffer_size: usize,
    pub device_name: Option<String>,
    pub latency_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UISettings {
    pub theme: String,
    pub module_grid_size: (u16, u16),
    pub show_parameter_values: bool,
    pub auto_save_interval: u64, // seconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    pub quit: String,
    pub save: String,
    pub load: String,
    pub add_module: String,
    pub delete_module: String,
    pub connect_modules: String,
    pub parameter_edit: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            audio: AudioSettings {
                sample_rate: 44100.0,
                buffer_size: 256,
                device_name: None,
                latency_ms: 10.0,
            },
            ui: UISettings {
                theme: "default".to_string(),
                module_grid_size: (80, 24),
                show_parameter_values: true,
                auto_save_interval: 300, // 5 minutes
            },
            keybindings: KeyBindings {
                quit: "q".to_string(),
                save: "s".to_string(),
                load: "l".to_string(),
                add_module: "a".to_string(),
                delete_module: "d".to_string(),
                connect_modules: "c".to_string(),
                parameter_edit: "Enter".to_string(),
            },
        }
    }
}

impl AppSettings {
    /// Load settings from default location
    pub fn load() -> Self {
        let config_path = Self::get_config_path();

        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(settings) => return settings,
                    Err(e) => eprintln!("Failed to parse settings: {}", e),
                },
                Err(e) => eprintln!("Failed to read settings file: {}", e),
            }
        }

        // Return default settings if loading fails
        Self::default()
    }

    /// Save settings to default location
    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = Self::get_config_path();

        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        fs::write(config_path, json)?;
        Ok(())
    }

    /// Get the path to the config file
    fn get_config_path() -> std::path::PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("maze-daw").join("settings.json")
        } else {
            std::path::PathBuf::from("settings.json")
        }
    }
}

/// Preset management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulePreset {
    pub name: String,
    pub module_type: crate::modules::ModuleType,
    pub parameters: HashMap<String, f32>,
    pub description: String,
    pub tags: Vec<String>,
}

impl ModulePreset {
    pub fn from_module(module: &AudioModule, name: &str) -> Self {
        let parameters: HashMap<String, f32> = module
            .parameters
            .iter()
            .map(|param| (param.name.clone(), param.value))
            .collect();

        Self {
            name: name.to_string(),
            module_type: module.module_type,
            parameters,
            description: String::new(),
            tags: Vec::new(),
        }
    }

    pub fn apply_to_module(&self, module: &mut AudioModule) -> anyhow::Result<()> {
        if module.module_type != self.module_type {
            return Err(anyhow::anyhow!("Module type mismatch"));
        }

        for param in &mut module.parameters {
            if let Some(&value) = self.parameters.get(&param.name) {
                param.value = value.clamp(param.min, param.max);
            }
        }

        Ok(())
    }
}

/// Preset bank for managing collections of presets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetBank {
    pub name: String,
    pub presets: Vec<ModulePreset>,
}

impl PresetBank {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            presets: Vec::new(),
        }
    }

    pub fn add_preset(&mut self, preset: ModulePreset) {
        self.presets.push(preset);
    }

    pub fn remove_preset(&mut self, name: &str) {
        self.presets.retain(|preset| preset.name != name);
    }

    pub fn get_preset(&self, name: &str) -> Option<&ModulePreset> {
        self.presets.iter().find(|preset| preset.name == name)
    }

    pub fn get_presets_for_type(
        &self,
        module_type: crate::modules::ModuleType,
    ) -> Vec<&ModulePreset> {
        self.presets
            .iter()
            .filter(|preset| preset.module_type == module_type)
            .collect()
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let bank: PresetBank = serde_json::from_str(&content)?;
        Ok(bank)
    }
}

/// Recent projects management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentProjects {
    pub projects: Vec<RecentProject>,
    pub max_entries: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentProject {
    pub name: String,
    pub path: std::path::PathBuf,
    pub last_opened: String,
}

impl Default for RecentProjects {
    fn default() -> Self {
        Self {
            projects: Vec::new(),
            max_entries: 10,
        }
    }
}

impl RecentProjects {
    pub fn add_project<P: AsRef<Path>>(&mut self, name: &str, path: P) {
        let path = path.as_ref().to_path_buf();
        let now = chrono::Utc::now().to_rfc3339();

        // Remove existing entry if present
        self.projects.retain(|p| p.path != path);

        // Add to front
        self.projects.insert(
            0,
            RecentProject {
                name: name.to_string(),
                path,
                last_opened: now,
            },
        );

        // Trim to max entries
        if self.projects.len() > self.max_entries {
            self.projects.truncate(self.max_entries);
        }
    }

    pub fn remove_project<P: AsRef<Path>>(&mut self, path: P) {
        let path = path.as_ref();
        self.projects.retain(|p| p.path != path);
    }

    pub fn get_projects(&self) -> &[RecentProject] {
        &self.projects
    }
}

/// Export/Import functionality
pub struct ProjectExporter;

impl ProjectExporter {
    /// Export project as a portable bundle
    pub fn export_bundle<P: AsRef<Path>>(
        project: &ProjectConfig,
        output_path: P,
    ) -> anyhow::Result<()> {
        // For now, just save as JSON
        // In the future, this could create a zip file with audio samples, etc.
        project.save_to_file(output_path)?;
        Ok(())
    }

    /// Import project from a bundle
    pub fn import_bundle<P: AsRef<Path>>(bundle_path: P) -> anyhow::Result<ProjectConfig> {
        // For now, just load JSON
        // In the future, this could extract from zip and handle audio samples
        ProjectConfig::load_from_file(bundle_path)
    }

    /// Export individual module as preset
    pub fn export_module_preset(module: &AudioModule, name: &str) -> ModulePreset {
        ModulePreset::from_module(module, name)
    }
}

/// Configuration validation
pub fn validate_project_config(config: &ProjectConfig) -> Vec<String> {
    let mut errors = Vec::new();

    // Check sample rate
    if config.sample_rate <= 0.0 || config.sample_rate > 192000.0 {
        errors.push("Invalid sample rate".to_string());
    }

    // Check buffer size
    if config.buffer_size == 0 || config.buffer_size > 4096 {
        errors.push("Invalid buffer size".to_string());
    }

    // Validate connections
    for conn in &config.connections {
        if !config.modules.contains_key(&conn.from_module) {
            errors.push(format!(
                "Connection references non-existent source module: {}",
                conn.from_module
            ));
        }
        if !config.modules.contains_key(&conn.to_module) {
            errors.push(format!(
                "Connection references non-existent destination module: {}",
                conn.to_module
            ));
        }
    }

    // Check for duplicate module names (warning, not error)
    let mut names = std::collections::HashSet::new();
    for module in config.modules.values() {
        if !names.insert(&module.name) {
            errors.push(format!("Duplicate module name: {}", module.name));
        }
    }

    errors
}
