mod tech;

use std::{collections::HashMap, fmt::Debug, fs, iter, path::Path};

use anyhow::{Context, Error};
use serde::Deserialize;
pub use tech::WappTech;

#[cfg(feature = "http")]
use http::HeaderMap;

#[derive(Debug)]
pub struct WappAnalyzer {
    pub groups: HashMap<i32, WappTechGroup>,
    pub cats: HashMap<i32, WappTechCategory>,
    pub techs: HashMap<String, WappTech>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WappTechGroup {
    #[serde(skip_deserializing)]
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WappTechCategory {
    #[serde(skip_deserializing)]
    pub id: i32,
    pub groups: Vec<i32>,
    pub name: String,
    pub priority: i32,
}

pub trait WappPage {
    fn url(&self) -> Option<&str> {
        None
    }

    #[cfg(feature = "http")]
    fn headers(&self) -> Option<&HeaderMap> {
        None
    }

    fn html(&self) -> Option<&str> {
        None
    }

    fn text(&self) -> Option<&str> {
        None
    }
}

#[derive(Debug)]
pub struct WappCheckResult {
    pub tech_name: String,
    pub confidence: i32,
    pub version: Option<String>,
}

impl WappAnalyzer {
    pub fn new_empty() -> Self {
        Self {
            cats: HashMap::new(),
            groups: HashMap::new(),
            techs: HashMap::new(),
        }
    }

    pub fn from_dir<P: AsRef<Path>>(data_dir: P) -> Result<Self, Error> {
        let path: &Path = data_dir.as_ref();

        let cat_file = path.join("categories.json");
        let group_file = path.join("groups.json");
        let tech_files = iter::once('_')
            .chain('a'..='z')
            .map(|c| path.join(format!("technologies/{c}.json")));

        Self::from_files(cat_file, group_file, tech_files)
    }

    pub fn from_files<P, I>(cat_file: P, group_file: P, tech_files: I) -> Result<Self, Error>
    where
        P: AsRef<Path> + Debug,
        I: Iterator<Item = P>,
    {
        let cat_bytes = fs::read(&cat_file).with_context(|| {
            let filename = cat_file.as_ref().to_string_lossy();
            format!("Failed to open file {filename}",)
        })?;

        let group_bytes = fs::read(&group_file).with_context(|| {
            let filename = group_file.as_ref().to_string_lossy();
            format!("Failed to open file {filename}",)
        })?;

        let mut tech_bytes_vec = Vec::new();
        for path in tech_files {
            let bytes = fs::read(&path).with_context(|| {
                let filename = path.as_ref().to_string_lossy();
                format!("Failed to open file {filename}",)
            })?;
            tech_bytes_vec.push(bytes);
        }
        let tech_bytes: Vec<&[u8]> = tech_bytes_vec.iter().map(|b| b.as_slice()).collect();

        Self::from_bytes(&cat_bytes, &group_bytes, &tech_bytes)
    }

    pub fn from_bytes(
        cat_bytes: &[u8],
        group_bytes: &[u8],
        tech_bytes: &[&[u8]],
    ) -> Result<Self, Error> {
        Ok(Self {
            groups: WappTechGroup::load_from_bytes(group_bytes)
                .context("Loading wapp technology groups")?,
            cats: WappTechCategory::load_from_bytes(cat_bytes)
                .context("Loading wapp technology categories")?,
            techs: {
                let mut techs = HashMap::new();
                for (i, data) in tech_bytes.iter().enumerate() {
                    techs.extend(
                        WappTech::load_from_bytes(data)
                            .with_context(|| format!("Loading wapp technology (file #{i})"))?,
                    )
                }
                techs
            },
        })
    }
}

impl WappTechGroup {
    pub(crate) fn load_from_bytes(bytes: &[u8]) -> Result<HashMap<i32, Self>, Error> {
        let data = serde_json::from_slice::<HashMap<&str, Self>>(bytes)
            .context("Failed to parse JSON from bytes")?;

        let mut result = HashMap::<i32, Self>::with_capacity(data.len());

        for (id, item) in data {
            let id = id
                .parse::<i32>()
                .with_context(|| format!("Group {} should has an interger ID", item.name))?;
            result.insert(id, Self { id, ..item });
        }

        Ok(result)
    }
}

impl WappTechCategory {
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

impl WappAnalyzer {
    pub fn check<P: WappPage>(&self, page: &P) -> Vec<WappCheckResult> {
        let mut result = Vec::new();

        for tech in self.techs.values() {
            if let Some(r) = tech.check(page) {
                result.push(WappCheckResult {
                    tech_name: tech.name.clone(),
                    confidence: r.confidence,
                    version: r.version,
                });
            }
        }

        result
    }
}
