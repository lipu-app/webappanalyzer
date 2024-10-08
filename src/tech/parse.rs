use std::{collections::HashMap, sync::OnceLock};

use anyhow::{anyhow, bail, Context, Error};
use regex::Regex;
use scraper::Selector;
use serde::Deserialize;

use super::{
    Tagged, WappTech, WappTechDomPatttern, WappTechPricing, WappTechVersionPattern,
    WappTechVersionValue,
};

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
    pub requires: Option<serde_json::Value>,
    pub requires_category: Option<serde_json::Value>,
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

/// Transform a `Option<serde_json::Value>` to a `Vec<T>` with `f`.
///
/// Mapping:
///
/// `None` => `vec![]`
///
/// `"str"` => `vec![f("str")]`
///
/// `[v1, v2, ...]` => `vec![f(v1), f(v2), ...]`
fn to_vec<T, F>(value: Option<serde_json::Value>, f: F) -> Vec<T>
where
    F: Fn(serde_json::Value) -> Result<T, Error>,
{
    match value {
        None => Vec::new(),
        Some(serde_json::Value::Array(a)) => a.into_iter().map(f).filter_map(|x| x.ok()).collect(),
        Some(x) => match f(x) {
            Ok(x) => vec![x],
            Err(_) => Vec::new(),
        },
    }
}

fn to_i32_vec(value: Option<serde_json::Value>) -> Vec<i32> {
    to_vec(value, |x| match x {
        serde_json::Value::Number(x) => match x.as_i64() {
            Some(x) => Ok(x as i32),
            None => Err(anyhow!("Expect an i32, found {x}")),
        },
        x => Err(anyhow!("Expect an i32, found {x}")),
    })
}

fn to_string_vec(value: Option<serde_json::Value>) -> Vec<String> {
    to_vec(value, |s| match s {
        serde_json::Value::String(s) => Ok(s),
        x => Err(anyhow!("Expect a string, found {x}")),
    })
}

fn to_tagged_string_vec(value: Option<serde_json::Value>) -> Vec<Tagged<String>> {
    to_vec(value, |s| match s {
        serde_json::Value::String(s) => Tagged::parse(&s, |t| Ok(t.to_string())),
        x => Err(anyhow!("Expect a string, found {x}")),
    })
}

fn to_pattern_vec(value: Option<serde_json::Value>) -> Vec<Tagged<Regex>> {
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
        Some(serde_json::Value::Object(o)) => Ok(o
            .into_iter()
            .map(|(k, v)| -> (String, Vec<Tagged<Regex>>) { (k, to_pattern_vec(Some(v))) })
            .collect()),
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
                    implies: to_tagged_string_vec(item.implies),
                    requires: to_string_vec(item.requires),
                    requires_category: to_i32_vec(item.requires_category),
                    excludes: to_string_vec(item.excludes),
                    cookies: to_pattern_map(item.cookies)?,
                    dom: item
                        .dom
                        .map(WappTechDomPatttern::from_json)
                        .unwrap_or_default(),
                    dns: (),
                    js: (),
                    headers: to_pattern_map(item.headers)?,
                    html: to_pattern_vec(item.html),
                    text: to_pattern_vec(item.text),
                    css: (),
                    probe: (),
                    robots: (),
                    url: to_pattern_vec(item.url),
                    xhr: (),
                    meta: to_pattern_map(item.meta)?,
                    script_src: to_pattern_vec(item.script_src),
                    scripts: to_pattern_vec(item.scripts),
                },
            );
        }

        Ok(result)
    }
}

impl Tagged<()> {
    fn new(confidence: i32, version: Option<WappTechVersionPattern>) -> Self {
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
        let mut version = None;

        for p in parts {
            let (k, v) = p.split_once(':').unwrap();
            match k {
                "confidence" => {
                    confidence = v.parse().unwrap();
                }
                "version" => {
                    version = Some(WappTechVersionPattern::parse(v)?);
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

impl WappTechVersionPattern {
    fn parse(input: &str) -> Result<Self, Error> {
        static RE: OnceLock<Regex> = OnceLock::new();
        let re = RE.get_or_init(|| Regex::new(r#"^([^?]*)\?([^:]*):(.*)$"#).unwrap());

        Ok(match re.captures(input) {
            Some(c) => {
                let cond_var = match WappTechVersionValue::parse(&c[1])? {
                    Some(WappTechVersionValue::Var(i)) => i,
                    Some(WappTechVersionValue::Const(s)) => {
                        bail!("Unexpected constant in condition: {s}")
                    }
                    None => {
                        bail!("Unexpected empty string in condition")
                    }
                };

                WappTechVersionPattern::Conditional {
                    cond_var,
                    true_expr: WappTechVersionValue::parse(&c[2])?,
                    false_expr: WappTechVersionValue::parse(&c[3])?,
                }
            }
            None => match WappTechVersionValue::parse(input)? {
                Some(v) => WappTechVersionPattern::Always(v),
                None => bail!("Empty version value"),
            },
        })
    }
}

impl WappTechVersionValue {
    fn parse(input: &str) -> Result<Option<Self>, Error> {
        if input.is_empty() {
            return Ok(None);
        }

        static RE: OnceLock<Regex> = OnceLock::new();
        let re = RE.get_or_init(|| Regex::new(r#"\\(\d+)"#).unwrap());

        Ok(Some(match re.captures(input) {
            Some(c) => {
                if c[0].len() != input.len() {
                    return Err(anyhow!("Failed to parse version value {input}"));
                }
                let cond_var = c[1]
                    .parse()
                    .context("Failed to parse version value as interger")?;
                Self::Var(cond_var)
            }
            None => Self::Const(input.into()),
        }))
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
            exists: Tagged::new(
                tagged_selector.confidence,
                tagged_selector.version,
            ),
            text: None,
            attributes: Vec::new(),
            properties: (),
        })
    }

    fn from_json(input: serde_json::Value) -> Vec<Self> {
        match input {
            serde_json::Value::String(s) => match Self::from_selector(&s) {
                Ok(x) => vec![x],
                Err(_) => Vec::new(),
            },
            serde_json::Value::Array(a) => {
                let mut vals = Vec::new();
                for x in a {
                    let s = match x {
                        serde_json::Value::String(s) => s,
                        _ => continue,
                    };
                    match Self::from_selector(&s) {
                        Ok(v) => vals.push(v),
                        Err(_) => continue,
                    };
                }
                vals
            }
            serde_json::Value::Object(o) => {
                let mut vals = Vec::new();
                for (selector, description) in o {
                    let mut pat = match Self::from_selector(&selector) {
                        Ok(p) => p,
                        Err(_) => continue,
                    };
                    let description = match description {
                        serde_json::Value::Object(d) => d,
                        _ => continue,
                    };
                    for (k, v) in description {
                        match k.as_str() {
                            "exists" => {
                                let s = match v {
                                    serde_json::Value::String(s) => s,
                                    _ => continue,
                                };
                                let t = Tagged::parse(&s, |t| {
                                    if !t.is_empty() {
                                        Err(anyhow!("Expect an empty string, found {t}"))?
                                    }
                                    Ok(())
                                });
                                match t {
                                    Ok(p) => pat.exists = p,
                                    Err(_) => continue,
                                };
                            }
                            "text" => {
                                pat.text = v
                                    .as_str()
                                    .ok_or_else(|| anyhow!("Expect string, fonud {v}"))
                                    .and_then(|t| {
                                        Tagged::<Regex>::parse(t, |s| {
                                            Regex::new(s).with_context(|| {
                                                format!("Failed parsing regular expression {s}")
                                            })
                                        })
                                    })
                                    .ok();
                            }
                            "attributes" | "properties" => {
                                if let Ok(x) = to_pattern_map(Some(v.clone())) {
                                    pat.attributes.extend(x);
                                }
                            }
                            "src" => {}
                            x => panic!("{x}"),
                        }
                    }
                    vals.push(pat);
                }
                vals
            }
            _ => panic!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{anyhow, Error};

    use super::{to_vec, Tagged, WappTechVersionPattern, WappTechVersionValue};

    #[test]
    fn test_to_vec() {
        use serde_json::json;

        assert_eq!(to_vec(None, |_| Ok(())), vec![]);

        assert_eq!(to_vec(Some(json!(1)), |x| Ok(x.as_i64().unwrap())), vec![1]);

        assert_eq!(
            to_vec(Some(json!(1)), |_| Err::<(), Error>(anyhow!("anyhow"))),
            vec![],
        );

        assert_eq!(
            to_vec(Some(json!([1, 2, 3])), |x| Ok(x.as_i64().unwrap())),
            vec![1, 2, 3],
        );

        assert_eq!(
            to_vec(
                Some(json!([1, 2, 3])),
                |x| if x.as_i64().unwrap() % 2 != 0 {
                    Ok(x)
                } else {
                    Err(anyhow!("anyhow"))
                }
            ),
            vec![1, 3],
        );
    }

    #[test]
    fn test_parse_tagged() {
        assert_eq!(
            Tagged::parse("pattern", |s| Ok(s.to_string())).unwrap(),
            Tagged {
                inner: "pattern".to_string(),
                confidence: 100,
                version: None,
            }
        );

        assert_eq!(
            Tagged::parse("pattern\\;confidence:80", |s| Ok(s.to_string())).unwrap(),
            Tagged {
                inner: "pattern".to_string(),
                confidence: 80,
                version: None,
            },
        );

        assert_eq!(
            Tagged::parse("(pattern)\\;version:\\1", |s| Ok(s.to_string())).unwrap(),
            Tagged {
                inner: "(pattern)".to_string(),
                confidence: 100,
                version: Some(WappTechVersionPattern::Always(WappTechVersionValue::Var(1))),
            },
        );

        assert_eq!(
            Tagged::parse("(pattern)\\;confidence:80\\;version:\\1?next:\\2", |s| Ok(
                s.to_string()
            ))
            .unwrap(),
            Tagged {
                inner: "(pattern)".to_string(),
                confidence: 80,
                version: Some(WappTechVersionPattern::Conditional {
                    cond_var: 1,
                    true_expr: Some(WappTechVersionValue::Const("next".into())),
                    false_expr: Some(WappTechVersionValue::Var(2)),
                }),
            },
        );
    }

    #[test]
    fn test_parse_wapp_tech_version_pattern() {
        assert_eq!(
            WappTechVersionPattern::parse("v1").unwrap(),
            WappTechVersionPattern::Always(WappTechVersionValue::Const("v1".into())),
        );

        assert_eq!(
            WappTechVersionPattern::parse("\\1").unwrap(),
            WappTechVersionPattern::Always(WappTechVersionValue::Var(1)),
        );

        assert_eq!(
            WappTechVersionPattern::parse("\\1?next:\\2").unwrap(),
            WappTechVersionPattern::Conditional {
                cond_var: 1,
                true_expr: Some(WappTechVersionValue::Const("next".into())),
                false_expr: Some(WappTechVersionValue::Var(2)),
            }
        );

        assert_eq!(
            WappTechVersionPattern::parse("\\1?\\1:legacy").unwrap(),
            WappTechVersionPattern::Conditional {
                cond_var: 1,
                true_expr: Some(WappTechVersionValue::Var(1)),
                false_expr: Some(WappTechVersionValue::Const("legacy".into())),
            }
        );

        assert!(WappTechVersionPattern::parse("conststr?\\1:\\2").is_err());
    }

    #[test]
    fn test_parse_wapp_tech_version_value() {
        assert_eq!(WappTechVersionValue::parse("").unwrap(), None);

        assert_eq!(
            WappTechVersionValue::parse("v1").unwrap(),
            Some(WappTechVersionValue::Const("v1".into())),
        );

        assert_eq!(
            WappTechVersionValue::parse("\\42").unwrap(),
            Some(WappTechVersionValue::Var(42)),
        );

        assert!(WappTechVersionValue::parse("left\\1right").is_err());
    }
}
