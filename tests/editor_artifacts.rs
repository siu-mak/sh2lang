use std::fs;

#[test]
fn test_vscode_package_json_valid() {
    let content = fs::read_to_string("editors/vscode/package.json")
        .expect("Failed to read package.json");
    
    // Verify it's valid JSON
    let _: serde_json::Value = serde_json::from_str(&content)
        .expect("package.json is not valid JSON");
}

#[test]
fn test_vscode_language_config_valid() {
    let content = fs::read_to_string("editors/vscode/language-configuration.json")
        .expect("Failed to read language-configuration.json");
    
    // Verify it's valid JSON
    let _: serde_json::Value = serde_json::from_str(&content)
        .expect("language-configuration.json is not valid JSON");
}

#[test]
fn test_textmate_grammar_valid() {
    let content = fs::read_to_string("editors/vscode/syntaxes/sh2.tmLanguage.json")
        .expect("Failed to read sh2.tmLanguage.json");
    
    // Verify it's valid JSON
    let _: serde_json::Value = serde_json::from_str(&content)
        .expect("sh2.tmLanguage.json is not valid JSON");
}

#[test]
fn test_ebnf_grammar_exists() {
    let content = fs::read_to_string("artifacts/grammar/sh2.ebnf")
        .expect("Failed to read sh2.ebnf");
    
    // Basic sanity check - should contain some expected tokens
    assert!(content.contains("program"));
    assert!(content.contains("statement"));
    assert!(content.contains("expression"));
}

#[test]
fn test_textmate_keywords_coverage() {
    use sh2c::lang_spec::KEYWORDS;
    
    let content = fs::read_to_string("editors/vscode/syntaxes/sh2.tmLanguage.json")
        .expect("Failed to read sh2.tmLanguage.json");
    
    let grammar: serde_json::Value = serde_json::from_str(&content)
        .expect("Invalid JSON");
    
    // Extract keyword pattern
    let keyword_pattern = grammar["repository"]["keywords"]["patterns"][0]["match"]
        .as_str()
        .expect("Missing keyword match pattern");
    
    // Check that all keywords appear in the pattern
    for keyword in KEYWORDS {
        assert!(
            keyword_pattern.contains(keyword),
            "Keyword '{}' not found in TextMate grammar pattern",
            keyword
        );
    }
}

#[test]
fn test_textmate_builtins_coverage() {
    use sh2c::lang_spec::BUILTINS;
    
    let content = fs::read_to_string("editors/vscode/syntaxes/sh2.tmLanguage.json")
        .expect("Failed to read sh2.tmLanguage.json");
    
    let grammar: serde_json::Value = serde_json::from_str(&content)
        .expect("Invalid JSON");
    
    // Extract builtin pattern
    let builtin_pattern = grammar["repository"]["builtins"]["patterns"][0]["match"]
        .as_str()
        .expect("Missing builtin match pattern");
    
    // Check that all builtins appear in the pattern
    for builtin in BUILTINS {
        assert!(
            builtin_pattern.contains(builtin),
            "Builtin '{}' not found in TextMate grammar pattern",
            builtin
        );
    }
}

#[test]
fn test_artifact_snapshots() {
    let update_snapshots = std::env::var("SH2C_UPDATE_SNAPSHOTS").is_ok();
    
    // Test TextMate grammar snapshot
    {
        let actual = fs::read_to_string("editors/vscode/syntaxes/sh2.tmLanguage.json")
            .expect("Failed to read sh2.tmLanguage.json");
        let expected_path = "tests/fixtures/editor_sh2.tmLanguage.json.expected";
        
        if update_snapshots {
            fs::write(expected_path, &actual)
                .expect("Failed to write snapshot");
        }
        
        let expected = fs::read_to_string(expected_path)
            .unwrap_or_default();
        
        assert_eq!(
            actual.trim(),
            expected.trim(),
            "TextMate grammar doesn't match snapshot"
        );
    }
    
    // Test language configuration snapshot
    {
        let actual = fs::read_to_string("editors/vscode/language-configuration.json")
            .expect("Failed to read language-configuration.json");
        let expected_path = "tests/fixtures/editor_language-configuration.json.expected";
        
        if update_snapshots {
            fs::write(expected_path, &actual)
                .expect("Failed to write snapshot");
        }
        
        let expected = fs::read_to_string(expected_path)
            .unwrap_or_default();
        
        assert_eq!(
            actual.trim(),
            expected.trim(),
            "Language configuration doesn't match snapshot"
        );
    }
    
    // Test EBNF grammar snapshot
    {
        let actual = fs::read_to_string("artifacts/grammar/sh2.ebnf")
            .expect("Failed to read sh2.ebnf");
        let expected_path = "tests/fixtures/editor_sh2.ebnf.expected";
        
        if update_snapshots {
            fs::write(expected_path, &actual)
                .expect("Failed to write snapshot");
        }
        
        let expected = fs::read_to_string(expected_path)
            .unwrap_or_default();
        
        assert_eq!(
            actual.trim(),
            expected.trim(),
            "EBNF grammar doesn't match snapshot"
        );
    }
}
