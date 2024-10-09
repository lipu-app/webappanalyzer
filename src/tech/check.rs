use regex::{Captures, Regex};

use crate::WappPage;

use super::{Tagged, WappTech, WappTechCheckResult, WappTechVersionPattern, WappTechVersionValue};

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

trait WappTechCheck<T> {
    fn check(&self, input: T) -> Option<WappTechCheckResult>;
}

impl WappTechCheck<&str> for Tagged<Regex> {
    fn check(&self, input: &str) -> Option<WappTechCheckResult> {
        let captures = self.inner.captures(input)?;

        Some(WappTechCheckResult {
            confidence: self.confidence,
            version: match &self.version {
                None => None,
                Some(WappTechVersionPattern::Always(s)) => Some(s.resolve(captures)),
                Some(WappTechVersionPattern::Conditional { cond_var, true_expr, false_expr }) => match captures.get(*cond_var) {
                    Some(_) => true_expr.resolve(captures),
                    None => false_expr.resolve(captures),
                }
            }
        })
    }
}

impl WappTechCheck<&str> for Vec<Tagged<Regex>> {
    fn check(&self, input: &str) -> Option<WappTechCheckResult> {
        let mut best_result: Option<WappTechCheckResult> = None;

        for pat in self {
            if let Some(result) = pat.check(input) {
                if result.confidence >= 100 {
                    return Some(result);
                }
                if result.confidence > best_result.as_ref().map(|x| x.confidence).unwrap_or(0) {
                    best_result = Some(result);
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

    pub fn check_html(&self, html: &str) -> Option<WappTechCheckResult> {
        self.html.check(html)
    }

    pub fn check_text(&self, text: &str) -> Option<WappTechCheckResult> {
        self.text.check(text)
    }

    pub fn check<P: WappPage>(&self, page: &P) -> Option<WappTechCheckResult> {
        let mut best_result: Option<WappTechCheckResult> = None;

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

        if let Some(url) = page.url() {
            handle_check_result!(self.check_url(url), best_result);
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
