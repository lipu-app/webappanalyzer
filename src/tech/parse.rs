use std::{collections::HashMap, fs, path::Path, sync::OnceLock};

use anyhow::{anyhow, bail, Context, Error};
use regex::Regex;
use scraper::Selector;
use serde::Deserialize;

use super::{Tagged, WappTech, WappTechDomPatttern, WappTechPricing, WappTechVersion};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WappTechRaw {
    pub cats: Vec<i32>,
    pub website: String,
    pub description: Option<String>,
    #[allow(dead_code)]
    pub icon: Option<serde_json::Value>,
    pub cpe: Option<String>,
    pub saas: Option<bool>,
    pub oss: Option<bool>,
    pub pricing: Option<Vec<WappTechPricing>>,
    pub cert_issuer: Option<String>,
    pub implies: Option<serde_json::Value>,
    pub requires: Option<Vec<String>>,
    pub requires_category: Option<i32>,
    pub excludes: Option<serde_json::Value>,
    pub cookies: Option<serde_json::Value>,
    pub dom: Option<serde_json::Value>,
    #[allow(dead_code)]
    pub dns: Option<serde_json::Value>,
    #[allow(dead_code)]
    pub js: Option<serde_json::Value>,
    pub headers: Option<serde_json::Value>,
    pub html: Option<serde_json::Value>,
    pub text: Option<serde_json::Value>,
    #[allow(dead_code)]
    pub css: Option<serde_json::Value>,
    #[allow(dead_code)]
    pub probe: Option<serde_json::Value>,
    #[allow(dead_code)]
    pub robots: Option<serde_json::Value>,
    #[allow(dead_code)]
    pub xhr: Option<serde_json::Value>,
    pub url: Option<serde_json::Value>,
    pub meta: Option<serde_json::Value>,
    pub script_src: Option<serde_json::Value>,
    pub scripts: Option<serde_json::Value>,
}

impl WappTech {
    pub(crate) fn load_from_file<P: AsRef<Path>>(path: &P) -> Result<HashMap<String, Self>, Error> {
        let bytes = fs::read(path)
            .with_context(|| format!("Failed to open file {}", path.as_ref().to_string_lossy()))?;

        Self::load_from_bytes(&bytes)
    }
}

/// Transform a `Option<serde_json::Value>` to a `Vec<T>` with `f`.
///
/// Mapping:
///
/// `None` => `vec![]`
///
/// `"str"` => `vec![f("str")]`
///
/// `[v1, v2, ...]` => `vec![f(v1), f(v2), ...]`
fn to_vec<T, F>(value: Option<serde_json::Value>, f: F) -> Result<Vec<T>, Error>
where
    F: Fn(serde_json::Value) -> Result<T, Error>,
{
    match value {
        None => Ok(Vec::new()),
        Some(serde_json::Value::Array(a)) => a.into_iter().map(f).collect(),
        Some(x) => f(x).map(|x| vec![x]),
    }
}

fn to_string_vec(value: Option<serde_json::Value>) -> Result<Vec<String>, Error> {
    to_vec(value, |s| match s {
        serde_json::Value::String(s) => Ok(s),
        x => Err(anyhow!("Expect a string, found {x}")),
    })
}

fn to_tagged_string_vec(value: Option<serde_json::Value>) -> Result<Vec<Tagged<String>>, Error> {
    to_vec(value, |s| match s {
        serde_json::Value::String(s) => Tagged::parse(&s, |t| Ok(t.to_string())),
        x => Err(anyhow!("Expect a string, found {x}")),
    })
}

fn to_pattern_vec(value: Option<serde_json::Value>) -> Result<Vec<Tagged<Regex>>, Error> {
    to_vec(value, |s| match s {
        serde_json::Value::String(s) => Tagged::parse(&s, |t| {
            Regex::new(t).with_context(|| format!("Failed parsing regular expresion {t}"))
        }),
        x => Err(anyhow!("Expect a string, found {x}")),
    })
}

#[allow(clippy::type_complexity)]
fn to_pattern_map(
    value: Option<serde_json::Value>,
) -> Result<Vec<(String, Vec<Tagged<Regex>>)>, Error> {
    match value {
        None => Ok(Vec::new()),
        Some(serde_json::Value::Object(o)) => o
            .into_iter()
            .map(|(k, v)| -> Result<(String, Vec<Tagged<Regex>>), Error> {
                Ok((k, to_pattern_vec(Some(v))?))
            })
            .collect(),
        Some(x) => Err(anyhow!("Expect a object, found {x}")),
    }
}

impl WappTech {
    pub(crate) fn load_from_bytes(bytes: &[u8]) -> Result<HashMap<String, Self>, Error> {
        let data = serde_json::from_slice::<HashMap<String, WappTechRaw>>(bytes)
            .context("Failed to parse JSON from bytes")?;

        let mut result = HashMap::<String, Self>::with_capacity(data.len());

        for (name, item) in data {
            result.insert(
                name.clone(),
                Self {
                    name,
                    cats: item.cats,
                    website: item.website,
                    description: item.description,
                    icon: (),
                    cpe: item.cpe,
                    saas: item.saas,
                    oss: item.oss,
                    pricing: item.pricing.unwrap_or_default(),
                    cert_issuer: item.cert_issuer,
                    implies: to_tagged_string_vec(item.implies)?,
                    requires: item.requires.unwrap_or_default(),
                    requires_category: item.requires_category,
                    excludes: to_string_vec(item.excludes)?,
                    cookies: to_pattern_map(item.cookies)?,
                    dom: item
                        .dom
                        .map(WappTechDomPatttern::from_json)
                        .transpose()?
                        .unwrap_or_default(),
                    dns: (),
                    js: (),
                    headers: to_pattern_map(item.headers)?,
                    html: to_pattern_vec(item.html)?,
                    text: to_pattern_vec(item.text)?,
                    css: (),
                    probe: (),
                    robots: (),
                    url: to_pattern_vec(item.url)?,
                    xhr: (),
                    meta: to_pattern_map(item.meta)?,
                    script_src: to_pattern_vec(item.script_src)?,
                    scripts: to_pattern_vec(item.scripts)?,
                },
            );
        }

        Ok(result)
    }
}

impl Tagged<()> {
    fn new(confidence: i32, version: WappTechVersion) -> Self {
        Self {
            inner: (),
            confidence,
            version,
        }
    }
}

impl<T> Tagged<T> {
    fn parse<F>(input: &str, inner_parser: F) -> Result<Self, Error>
    where
        F: FnOnce(&str) -> Result<T, Error>,
    {
        let mut parts = input.split("\\;");

        let inner_input = parts.next().unwrap();
        let mut confidence = 100;
        let mut version = WappTechVersion::Unknown;

        for p in parts {
            let (k, v) = p.split_once(':').unwrap();
            match k {
                "confidence" => {
                    confidence = v.parse().unwrap();
                }
                "version" => {
                    static RE: OnceLock<Regex> = OnceLock::new();
                    let re = RE.get_or_init(|| Regex::new(r#"^([^?]*)\?([^:]*):(.*)$"#).unwrap());

                    version = match re.captures(v) {
                        None => WappTechVersion::Always(v.into()),
                        Some(c) => {
                            WappTechVersion::Conditional(c[1].into(), c[2].into(), c[3].into())
                        }
                    }
                }
                tag => bail!("Unknown tag name: {}", tag),
            }
        }

        Ok(Self {
            inner: inner_parser(inner_input).context("Failed to parse content without tag")?,
            confidence,
            version,
        })
    }
}

impl WappTechDomPatttern {
    fn from_selector(input: &str) -> Result<Self, Error> {
        let tagged_selector = Tagged::<Selector>::parse(input, |s| {
            Selector::parse(s)
                .map_err(|e| anyhow!(format!("Failed to parse html selector {s}: {e}")))
        })?;

        Ok(Self {
            selector: tagged_selector.inner,
            exists: Some(Tagged::new(
                tagged_selector.confidence,
                tagged_selector.version,
            )),
            text: None,
            attributes: Vec::new(),
            properties: (),
        })
    }

    fn from_json(input: serde_json::Value) -> Result<Vec<Self>, Error> {
        match input {
            serde_json::Value::String(s) => Self::from_selector(&s).map(|x| vec![x]),
            serde_json::Value::Array(a) => {
                let mut vals = Vec::new();
                for x in a {
                    let s = match x {
                        serde_json::Value::String(s) => s,
                        _ => Err(anyhow!("Expect a string, found {x}"))?,
                    };
                    vals.push(Self::from_selector(&s)?);
                }
                Ok(vals)
            }
            serde_json::Value::Object(o) => {
                let mut vals = Vec::new();
                for (selector, description) in o {
                    let mut pat = Self::from_selector(&selector)?;
                    let description = match description {
                        serde_json::Value::Object(d) => d,
                        x => Err(anyhow!("Expect an object, found {x}"))?,
                    };
                    for (k, v) in description {
                        match k.as_str() {
                            "exists" => {
                                let s = match v {
                                    serde_json::Value::String(s) => s,
                                    _ => Err(anyhow!("Expect a string, found {v}"))?,
                                };
                                pat.exists = Some(Tagged::parse(&s, |t| {
                                    if !t.is_empty() {
                                        Err(anyhow!("Expect an empty string, found {t}"))?
                                    }
                                    Ok(())
                                })?);
                            }
                            "text" => {
                                pat.text = Some(
                                    v.as_str()
                                        .ok_or_else(|| anyhow!("Expect string, fonud {v}"))
                                        .and_then(|t| {
                                            Tagged::<Regex>::parse(t, |s| {
                                                Regex::new(s).with_context(|| {
                                                    format!("Failed parsing regular expression {s}")
                                                })
                                            })
                                        })?,
                                );
                            }
                            "attributes" | "properties" => {
                                pat.attributes.extend(to_pattern_map(Some(v.clone()))?);
                            }
                            "src" => {}
                            x => panic!("{x}"),
                        }
                    }
                    vals.push(pat);
                }
                Ok(vals)
            }
            _ => panic!(),
        }
    }
}
