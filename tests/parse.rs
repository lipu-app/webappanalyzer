use std::path::PathBuf;

use webappanalyzer::WappAnalyzer;

fn test_parse_dataset(dataset: &str) {
    let dir = PathBuf::from_iter(["./tests/webappanalyzer-data", dataset, "src"]);
    let wapp_analyzer = WappAnalyzer::from_dir(dir).unwrap();

    assert!(!wapp_analyzer.cats.is_empty());
    assert!(!wapp_analyzer.groups.is_empty());
    assert!(!wapp_analyzer.techs.is_empty());
}

#[test]
#[ignore]
fn test_parse_wappalyzer_mit() {
    test_parse_dataset("wappalyzer-mit");
}

#[test]
#[ignore]
fn test_parse_wappalyzer() {
    test_parse_dataset("wappalyzer");
}

#[test]
#[ignore]
fn test_parse_webappanalyzer() {
    test_parse_dataset("webappanalyzer");
}
