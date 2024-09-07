use std::{collections::HashMap, fs, path::Path};

use anyhow::{Context, Error};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WappTechCategory {
    #[serde(skip_deserializing)]
    pub id: i32,
    pub groups: Vec<i32>,
    pub name: String,
    pub priority: i32,
}

impl WappTechCategory {
    pub(crate) fn load_from_file<P: AsRef<Path>>(path: P) -> Result<HashMap<i32, Self>, Error> {
        let bytes = fs::read(&path)
            .with_context(|| format!("Failed to open file {}", path.as_ref().to_string_lossy()))?;

        Self::load_from_bytes(&bytes)
    }

    pub(crate) fn load_from_bytes(bytes: &[u8]) -> Result<HashMap<i32, Self>, Error> {
        let data = serde_json::from_slice::<HashMap<&str, Self>>(bytes)
            .context("Failed to parse JSON from bytes")?;

        let mut result = HashMap::<i32, WappTechCategory>::with_capacity(data.len());

        for (id, item) in data {
            let id = id
                .parse::<i32>()
                .with_context(|| format!("Category {} should has an interger ID", item.name))?;
            result.insert(id, Self { id, ..item });
        }

        Ok(result)
    }
}
