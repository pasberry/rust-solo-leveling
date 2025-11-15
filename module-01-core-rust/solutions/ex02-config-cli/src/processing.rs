use std::collections::HashMap;
use std::fs;

use crate::config::Config;
use crate::error::ProcessingError;

#[derive(Debug)]
pub struct ProcessingResult {
    pub count: usize,
    pub output: String,
}

/// Process data according to configuration
pub fn process(config: &Config) -> Result<ProcessingResult, ProcessingError> {
    // Read input file
    let content = fs::read_to_string(&config.input.file)?;
    let lines: Vec<&str> = content.lines().collect();

    // Process based on operation
    let result = match config.processing.operation.as_str() {
        "filter" => process_filter(&lines, config)?,
        "transform" => process_transform(&lines, config)?,
        "count" => process_count(&lines, config)?,
        op => {
            return Err(ProcessingError::InvalidOperation(format!(
                "Unknown operation: {}",
                op
            )))
        }
    };

    // Write output
    fs::write(&config.output.file, &result.output)?;

    Ok(result)
}

fn process_filter(lines: &[&str], config: &Config) -> Result<ProcessingResult, ProcessingError> {
    let pattern = config.processing.pattern.as_ref().ok_or_else(|| {
        ProcessingError::MissingPattern("filter".to_string())
    })?;

    let filtered: Vec<&str> = if config.processing.case_sensitive {
        lines
            .iter()
            .filter(|line| line.contains(pattern))
            .copied()
            .collect()
    } else {
        let pattern_lower = pattern.to_lowercase();
        lines
            .iter()
            .filter(|line| line.to_lowercase().contains(&pattern_lower))
            .copied()
            .collect()
    };

    let output = filtered.join("\n");
    let count = filtered.len();

    Ok(ProcessingResult { count, output })
}

fn process_transform(
    lines: &[&str],
    config: &Config,
) -> Result<ProcessingResult, ProcessingError> {
    let transform_type = config.processing.transform.as_ref().ok_or_else(|| {
        ProcessingError::InvalidOperation("Transform requires transform type".to_string())
    })?;

    let transformed: Vec<String> = match transform_type.as_str() {
        "uppercase" => lines.iter().map(|line| line.to_uppercase()).collect(),
        "lowercase" => lines.iter().map(|line| line.to_lowercase()).collect(),
        "reverse" => lines
            .iter()
            .map(|line| line.chars().rev().collect())
            .collect(),
        t => {
            return Err(ProcessingError::InvalidOperation(format!(
                "Unknown transform: {}",
                t
            )))
        }
    };

    let output = transformed.join("\n");
    let count = transformed.len();

    Ok(ProcessingResult { count, output })
}

fn process_count(lines: &[&str], config: &Config) -> Result<ProcessingResult, ProcessingError> {
    let mut word_counts: HashMap<String, usize> = HashMap::new();

    for line in lines {
        for word in line.split_whitespace() {
            let word = if config.processing.case_sensitive {
                word.to_string()
            } else {
                word.to_lowercase()
            };

            *word_counts.entry(word).or_insert(0) += 1;
        }
    }

    // Sort by count descending
    let mut counts: Vec<_> = word_counts.into_iter().collect();
    counts.sort_by(|a, b| b.1.cmp(&a.1));

    let output = counts
        .iter()
        .map(|(word, count)| format!("{}: {}", word, count))
        .collect::<Vec<_>>()
        .join("\n");

    let count = counts.len();

    Ok(ProcessingResult { count, output })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_case_insensitive() {
        let lines = vec!["ERROR: failed", "INFO: success", "error: timeout"];
        let config = Config {
            input: crate::config::InputConfig {
                file: "".to_string(),
                format: "lines".to_string(),
            },
            processing: crate::config::ProcessingConfig {
                operation: "filter".to_string(),
                pattern: Some("ERROR".to_string()),
                case_sensitive: false,
                transform: None,
            },
            output: crate::config::OutputConfig {
                file: "".to_string(),
                format: "text".to_string(),
            },
        };

        let result = process_filter(&lines, &config).unwrap();
        assert_eq!(result.count, 2);
        assert!(result.output.contains("ERROR: failed"));
        assert!(result.output.contains("error: timeout"));
    }

    #[test]
    fn test_transform_uppercase() {
        let lines = vec!["hello", "world"];
        let config = Config {
            input: crate::config::InputConfig {
                file: "".to_string(),
                format: "lines".to_string(),
            },
            processing: crate::config::ProcessingConfig {
                operation: "transform".to_string(),
                pattern: None,
                case_sensitive: false,
                transform: Some("uppercase".to_string()),
            },
            output: crate::config::OutputConfig {
                file: "".to_string(),
                format: "text".to_string(),
            },
        };

        let result = process_transform(&lines, &config).unwrap();
        assert_eq!(result.output, "HELLO\nWORLD");
    }

    #[test]
    fn test_count_words() {
        let lines = vec!["hello world", "hello rust", "goodbye world"];
        let config = Config {
            input: crate::config::InputConfig {
                file: "".to_string(),
                format: "lines".to_string(),
            },
            processing: crate::config::ProcessingConfig {
                operation: "count".to_string(),
                pattern: None,
                case_sensitive: false,
                transform: None,
            },
            output: crate::config::OutputConfig {
                file: "".to_string(),
                format: "text".to_string(),
            },
        };

        let result = process_count(&lines, &config).unwrap();
        assert!(result.output.contains("hello: 2"));
        assert!(result.output.contains("world: 2"));
    }
}
