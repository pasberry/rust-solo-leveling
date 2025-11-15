use serde::Deserialize;
use std::fs;
use std::path::Path;

use crate::error::ConfigError;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub input: InputConfig,
    pub processing: ProcessingConfig,
    pub output: OutputConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct InputConfig {
    pub file: String,
    #[serde(default = "default_format")]
    pub format: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProcessingConfig {
    pub operation: String,
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default)]
    pub case_sensitive: bool,
    #[serde(default)]
    pub transform: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OutputConfig {
    pub file: String,
    #[serde(default = "default_format")]
    pub format: String,
}

fn default_format() -> String {
    "text".to_string()
}

impl Config {
    /// Load config from a file (TOML or JSON based on extension)
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;

        let config = match path.extension().and_then(|s| s.to_str()) {
            Some("toml") => toml::from_str(&content)?,
            Some("json") => serde_json::from_str(&content)?,
            Some(ext) => {
                return Err(ConfigError::Invalid(format!(
                    "Unsupported extension: {}",
                    ext
                )))
            }
            None => return Err(ConfigError::UnsupportedExtension),
        };

        Ok(config)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate operation
        let valid_operations = ["filter", "transform", "count"];
        if !valid_operations.contains(&self.processing.operation.as_str()) {
            return Err(ConfigError::Invalid(format!(
                "Invalid operation: {}. Must be one of: {}",
                self.processing.operation,
                valid_operations.join(", ")
            )));
        }

        // Validate that filter operation has a pattern
        if self.processing.operation == "filter" && self.processing.pattern.is_none() {
            return Err(ConfigError::Invalid(
                "Filter operation requires a pattern".to_string(),
            ));
        }

        // Validate transform operation has transform type
        if self.processing.operation == "transform" && self.processing.transform.is_none() {
            return Err(ConfigError::Invalid(
                "Transform operation requires a transform type".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_toml_config() {
        use std::io::Write;

        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("config.toml");

        let mut file = std::fs::File::create(&file_path).unwrap();
        writeln!(
            file,
            r#"
[input]
file = "data.txt"
format = "lines"

[processing]
operation = "filter"
pattern = "error"
case_sensitive = false

[output]
file = "output.txt"
format = "text"
"#
        )
        .unwrap();
        drop(file);

        let config = Config::load(&file_path).unwrap();
        assert_eq!(config.input.file, "data.txt");
        assert_eq!(config.processing.operation, "filter");
        assert_eq!(config.processing.pattern, Some("error".to_string()));
    }

    #[test]
    fn test_validate_missing_pattern() {
        let config = Config {
            input: InputConfig {
                file: "input.txt".to_string(),
                format: "lines".to_string(),
            },
            processing: ProcessingConfig {
                operation: "filter".to_string(),
                pattern: None,
                case_sensitive: false,
                transform: None,
            },
            output: OutputConfig {
                file: "output.txt".to_string(),
                format: "text".to_string(),
            },
        };

        assert!(config.validate().is_err());
    }
}
