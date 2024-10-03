mod parse;

use regex::Regex;
use scraper::Selector;
use serde::Deserialize;

#[derive(Debug)]
pub struct WappTech {
    pub name: String,
    /// One or more categories.
    pub cats: Vec<i32>,
    /// URL of the application's website.
    pub website: String,
    /// A short description of the technology in British English (max. 250 characters). Write in a neutral, factual
    /// tone; not like an ad.
    pub description: Option<String>,
    /// Application icon filename.
    #[allow(dead_code)]
    pub icon: (),
    /// [CPE](https://nvd.nist.gov/products/cpe) is a structured naming scheme for technologies. To check if a CPE is
    /// valid and exists (using v2.3), use the [search](https://nvd.nist.gov/products/cpe/search).
    pub cpe: Option<String>,
    /// The technology is offered as a Software-as-a-Service (SaaS), i.e. hosted or cloud-based.
    pub saas: Option<bool>,
    /// The technology has an open-source license.
    pub oss: Option<bool>,
    /// Cost indicator (based on a typical plan or average monthly price) and available pricing models. For paid
    /// products only.
    pub pricing: Vec<WappTechPricing>,
    pub cert_issuer: Option<String>,
    /// The presence of one application can imply the presence of another, e.g. WordPress means PHP is also in use.
    pub implies: Vec<Tagged<String>>,
    /// Similar to implies but detection only runs if the required technology has been identified. Useful for themes for
    /// a specific CMS.
    pub requires: Vec<String>,
    /// Similar to requires; detection only runs if a technology in the required category has been identified.
    pub requires_category: Vec<i32>,
    /// Opposite of implies. The presence of one application can exclude the presence of another.
    pub excludes: Vec<String>,
    /// Cookies.
    pub cookies: Vec<(String, Vec<Tagged<Regex>>)>,
    /// Uses a [query selector](https://developer.mozilla.org/en-US/docs/Web/API/Document/querySelectorAll) to inspect
    /// element properties, attributes and text content.
    pub dom: Vec<WappTechDomPatttern>,
    #[allow(dead_code)]
    pub dns: (),
    /// JavaScript properties (case sensitive). Avoid short property names to prevent matching minified code.
    #[allow(dead_code)]
    pub js: (),
    /// HTTP response headers.
    pub headers: Vec<(String, Vec<Tagged<Regex>>)>,
    /// HTML source code. Patterns must include an HTML opening tag to avoid matching plain text. For performance
    /// reasons, avoid `html` where possible and use `dom` instead.
    pub html: Vec<Tagged<Regex>>,
    /// Matches plain text. Should only be used in very specific cases where other methods can't be used.
    pub text: Vec<Tagged<Regex>>,
    /// CSS rules. Unavailable when a website enforces a same-origin policy. For performance reasons, only a portion of
    /// the available CSS rules are used to find matches.
    #[allow(dead_code)]
    pub css: (),
    /// Request a URL to test for its existence or match text content (NPM driver only).
    #[allow(dead_code)]
    pub probe: (),
    /// Robots.txt contents.
    #[allow(dead_code)]
    pub robots: (),
    /// Full URL of the page.
    pub url: Vec<Tagged<Regex>>,
    /// Hostnames of XHR requests.
    #[allow(dead_code)]
    pub xhr: (),
    /// HTML meta tags, e.g. generator.
    pub meta: Vec<(String, Vec<Tagged<Regex>>)>,
    /// URLs of JavaScript files included on the page.
    pub script_src: Vec<Tagged<Regex>>,
    /// JavaScript source code. Inspects inline and external scripts. For performance reasons, avoid `scripts` where
    /// possible and use `js` instead.
    pub scripts: Vec<Tagged<Regex>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
/// Cost indicator (based on a typical plan or average monthly price) and available pricing models. For paid products
/// only.
pub enum WappTechPricing {
    /// Less than US $100 / mo
    Low,
    /// Between US $100 - $1,000 / mo
    Mid,
    /// More than US $1,000 / mo
    High,
    /// Free plan available
    Freemium,
    /// One-time payments accepted
    Onetime,
    /// Subscriptions available
    Recurring,
    /// Price on asking
    Poa,
    /// Pay as you go (e.g. commissions or usage-based fees)
    Payg,
}

#[derive(Debug)]
pub struct WappTechDomPatttern {
    selector: Selector,
    exists: Option<Tagged<()>>,
    text: Option<Tagged<Regex>>,
    attributes: Vec<(String, Vec<Tagged<Regex>>)>,
    #[allow(dead_code)]
    properties: (),
}

/// Tags (a non-standard syntax) can be appended to patterns (and implies and excludes, separated by \\;) to store
/// additional information.
#[derive(Debug)]
pub struct Tagged<T> {
    pub inner: T,

    /// Indicates a less reliable pattern that may cause false positives. The aim is to achieve a combined confidence of
    /// 100%. Defaults to 100% if not specified.
    pub confidence: i32,

    /// Gets the version number from a pattern match using a special syntax.
    pub version: WappTechVersion,
}

#[derive(Debug)]
pub enum WappTechVersion {
    Unknown,
    Always(String),
    Conditional(String, String, String),
}
