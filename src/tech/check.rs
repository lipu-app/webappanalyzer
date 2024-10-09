use regex::{Captures, Regex};

use crate::WappPage;

use super::{Tagged, WappTech, WappTechCheckResult, WappTechVersionPattern, WappTechVersionValue};

#[cfg(feature = "cookie")]
use cookie::Cookie;

#[cfg(feature = "http")]
use http::{HeaderMap, HeaderValue};

#[cfg(feature = "scraper")]
use scraper::Html;

#[cfg(feature = "scraper")]
use super::WappTechDomPatttern;

trait ResolveVersion {
    type Version;

    fn resolve(&self, captures: Captures) -> Self::Version;
}

impl ResolveVersion for WappTechVersionValue {
    type Version = String;

    fn resolve(&self, captures: Captures) -> Self::Version {
        match self {
            WappTechVersionValue::Const(s) => s.clone(),
            WappTechVersionValue::Var(i) => captures[*i].into(),
        }
    }
}

impl ResolveVersion for Option<WappTechVersionValue> {
    type Version = Option<String>;

    fn resolve(&self, captures: Captures) -> Self::Version {
        self.as_ref().map(|x| x.resolve(captures))
    }
}

macro_rules! handle_check_result {
    ($check_call:expr, $best_result:ident) => {
        if let Some(__result) = $check_call {
            if __result.confidence >= 100 {
                return Some(__result);
            }
            if __result.confidence > $best_result.as_ref().map(|x| x.confidence).unwrap_or(0) {
                $best_result = Some(__result);
            }
        }
    };
}

trait WappTechCheck<T> {
    fn check(&self, input: T) -> Option<WappTechCheckResult>;
}

impl WappTechCheck<()> for Tagged<()> {
    fn check(&self, _input: ()) -> Option<WappTechCheckResult> {
        Some(WappTechCheckResult {
            confidence: self.confidence,
            version: match &self.version {
                Some(WappTechVersionPattern::Always(WappTechVersionValue::Const(s))) => Some(s.clone()),
                Some(_) => unreachable!(),
                None => None,
            },
        })
    }
}

impl WappTechCheck<&str> for Tagged<Regex> {
    fn check(&self, input: &str) -> Option<WappTechCheckResult> {
        let captures = self.inner.captures(input)?;

        Some(WappTechCheckResult {
            confidence: self.confidence,
            version: match &self.version {
                None => None,
                Some(WappTechVersionPattern::Always(s)) => Some(s.resolve(captures)),
                Some(WappTechVersionPattern::Conditional {
                    cond_var,
                    true_expr,
                    false_expr,
                }) => match captures.get(*cond_var) {
                    Some(_) => true_expr.resolve(captures),
                    None => false_expr.resolve(captures),
                },
            },
        })
    }
}

impl<P, T> WappTechCheck<T> for Vec<P>
where
    P: WappTechCheck<T>,
    T: Copy,
{
    fn check(&self, input: T) -> Option<WappTechCheckResult> {
        let mut best_result: Option<WappTechCheckResult> = None;

        for pat in self {
            handle_check_result!(pat.check(input), best_result);
        }

        best_result
    }
}

#[cfg(feature = "http")]
impl WappTechCheck<&HeaderValue> for Tagged<Regex> {
    fn check(&self, input: &HeaderValue) -> Option<WappTechCheckResult> {
        self.check(input.to_str().ok()?)
    }
}

#[cfg(feature = "http")]
impl WappTechCheck<&HeaderMap> for Vec<(String, Vec<Tagged<Regex>>)> {
    fn check(&self, input: &HeaderMap) -> Option<WappTechCheckResult> {
        let mut best_result: Option<WappTechCheckResult> = None;

        for (header_key, header_value) in input {
            for (pat_key, pat) in self {
                if pat_key.eq_ignore_ascii_case(header_key.as_str()) {
                    handle_check_result!(pat.check(header_value), best_result);
                }
            }
        }

        best_result
    }
}

#[cfg(feature = "cookie")]
impl<'c> WappTechCheck<&[Cookie<'c>]> for Vec<(String, Vec<Tagged<Regex>>)> {
    fn check(&self, input: &[Cookie]) -> Option<WappTechCheckResult> {
        let mut best_result: Option<WappTechCheckResult> = None;

        for cookie in input {
            for (pat_key, pat) in self {
                if pat_key == cookie.name() {
                    handle_check_result!(pat.check(cookie.value()), best_result);
                }
            }
        }

        best_result
    }
}

#[cfg(feature = "scraper")]
impl WappTechCheck<&Html> for WappTechDomPatttern {
    fn check(&self, input: &Html) -> Option<WappTechCheckResult> {
        let mut best_result: Option<WappTechCheckResult> = None;

        for el in input.select(&self.selector) {
            handle_check_result!(self.exists.check(()), best_result);

            for (attr_pat_key, attr_pat) in &self.attributes {
                if let Some(attr_value) = el.attr(attr_pat_key) {
                    handle_check_result!(attr_pat.check(attr_value), best_result);
                }
            }
        }

        best_result
    }
}

impl WappTech {
    pub fn check_url(&self, url: &str) -> Option<WappTechCheckResult> {
        self.url.check(url)
    }

    #[cfg(feature = "http")]
    pub fn check_headers(&self, headers: &HeaderMap) -> Option<WappTechCheckResult> {
        self.headers.check(headers)
    }

    #[cfg(feature = "cookie")]
    pub fn check_cookies(&self, cookies: &[Cookie]) -> Option<WappTechCheckResult> {
        self.cookies.check(cookies)
    }

    #[cfg(feature = "scraper")]
    pub fn check_dom(&self, dom: &Html) -> Option<WappTechCheckResult> {
        self.dom.check(dom)
    }

    pub fn check_html(&self, html: &str) -> Option<WappTechCheckResult> {
        self.html.check(html)
    }

    pub fn check_text(&self, text: &str) -> Option<WappTechCheckResult> {
        self.text.check(text)
    }

    pub fn check<P: WappPage>(&self, page: &P) -> Option<WappTechCheckResult> {
        let mut best_result: Option<WappTechCheckResult> = None;

        if let Some(url) = page.url() {
            handle_check_result!(self.check_url(url), best_result);
        }
        #[cfg(feature = "http")]
        if let Some(headers) = page.headers() {
            handle_check_result!(self.check_headers(headers), best_result);
        }
        #[cfg(feature = "cookie")]
        if let Some(cookies) = page.cookies() {
            handle_check_result!(self.check_cookies(cookies), best_result);
        }
        #[cfg(feature = "scraper")]
        if let Some(dom) = page.dom() {
            handle_check_result!(self.check_dom(dom), best_result);
        }
        if let Some(html) = page.html() {
            handle_check_result!(self.check_html(html), best_result);
        }
        if let Some(text) = page.text() {
            handle_check_result!(self.check_text(text), best_result);
        }

        best_result
    }
}
