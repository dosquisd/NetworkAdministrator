use clap::ValueEnum;

use crate::schemas::ArpResponse;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_tracing_level(&self) -> tracing::Level {
        match self {
            LogLevel::Trace => tracing::Level::TRACE,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LogFormat {
    // Human-readable format with colors
    Pretty,

    // JSON format for machine parsing
    Json,

    // Compact single-line format
    Compact,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Txt,
    Json,
    Csv,
    Yaml,
}

impl OutputFormat {
    pub fn to_string(&self) -> &'static str {
        match self {
            OutputFormat::Txt => "TXT",
            OutputFormat::Json => "JSON",
            OutputFormat::Csv => "CSV",
            OutputFormat::Yaml => "YAML",
        }
    }

    pub fn show_scanning_results(&self, results: &Vec<ArpResponse>) {
        match self {
            OutputFormat::Txt => {
                // Implement table output
                for res in results {
                    println!("IP: {}, MAC: {}, ALIAS: {}", res.target_ip, res.target_mac, res.alias.as_deref().unwrap_or("N/A"));
                }
            }
            OutputFormat::Json => {
                // Implement JSON output
                let json = serde_json::to_string_pretty(&results).unwrap();
                println!("{}", json);
            }
            OutputFormat::Csv => {
                // Implement CSV output
                println!("IP,MAC,ALIAS");
                for res in results {
                    println!("{},{},{}", res.target_ip, res.target_mac, res.alias.as_deref().unwrap_or("N/A"));
                }
            }
            OutputFormat::Yaml => {
                // Implement YAML output
                let yaml = serde_yaml::to_string(&results).unwrap();
                println!("{}", yaml);
            }
        }
    }
}
