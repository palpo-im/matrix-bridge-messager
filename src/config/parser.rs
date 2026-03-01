use anyhow::{Context, Result};
use config::{Config as ConfigRs, Environment, File};
use std::path::Path;

pub fn parse_config_file<P: AsRef<Path>>(path: P) -> Result<crate::config::Config> {
    let path = path.as_ref();

    let mut config_builder = ConfigRs::builder().add_source(File::from(path));

    config_builder = config_builder.add_source(
        Environment::with_prefix("MATRIX_BRIDGE_MESSAGE")
            .separator("__")
            .try_parsing(true),
    );

    let config = config_builder
        .build()
        .with_context(|| format!("Failed to build configuration from {:?}", path))?;

    let config: crate::config::Config = config
        .try_deserialize()
        .with_context(|| "Failed to deserialize configuration")?;

    crate::config::validator::validate_config(&config)?;

    Ok(config)
}
