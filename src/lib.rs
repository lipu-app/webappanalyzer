mod category;
mod group;
mod tech;

use std::{collections::HashMap, fmt::Debug, iter, path::Path};

use anyhow::{Context, Error};
pub use category::WappTechCategory;
pub use group::WappTechGroup;
pub use tech::WappTech;

#[derive(Debug)]
pub struct WappAnalyzer {
    pub cats: HashMap<i32, WappTechCategory>,
    pub groups: HashMap<i32, WappTechGroup>,
    pub techs: HashMap<String, WappTech>,
}

pub trait WappPage {
    fn url(&self) -> Option<&str> {
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
        Ok(Self {
            cats: WappTechCategory::load_from_file(cat_file)
                .context("Loading wapp technology categories")?,
            groups: WappTechGroup::load_from_file(group_file)
                .context("Loading wapp technology groups")?,
            techs: {
                let mut techs = HashMap::new();
                for path in tech_files {
                    techs.extend(
                        WappTech::load_from_file(&path)
                            .with_context(|| format!("Loading {path:?}"))?,
                    )
                }
                techs
            },
        })
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
