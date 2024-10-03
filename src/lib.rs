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
    pub tech: HashMap<String, WappTech>,
}

impl WappAnalyzer {
    pub fn new_empty() -> Self {
        Self {
            cats: HashMap::new(),
            groups: HashMap::new(),
            tech: HashMap::new(),
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
            tech: {
                let mut tech = HashMap::new();
                for path in tech_files {
                    tech.extend(
                        WappTech::load_from_file(&path)
                            .with_context(|| format!("Loading {path:?}"))?,
                    )
                }
                tech
            },
        })
    }
}
