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

/// Extract exact tokens from a TextMate regex pattern like \b(token1|token2|token3)\b
fn extract_tokens_from_pattern(pattern: &str) -> Vec<String> {
    // Find the first capturing group content (inside parentheses)
    let start = pattern.find('(').expect("Pattern should contain parentheses");
    let end = pattern.rfind(')').expect("Pattern should contain parentheses");
    let inner = &pattern[start + 1..end];
    
    // Split by | and collect tokens
    inner.split('|').map(|s| s.to_string()).collect()
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
    
    // Extract exact tokens from pattern
    let pattern_tokens = extract_tokens_from_pattern(keyword_pattern);
    
    // Check for duplicates
    let mut seen = std::collections::HashSet::new();
    for token in &pattern_tokens {
        assert!(
            seen.insert(token),
            "Duplicate token '{}' found in keyword pattern",
            token
        );
    }
    
    // Check that all keywords appear as exact tokens (not substrings)
    for keyword in KEYWORDS {
        assert!(
            pattern_tokens.contains(&keyword.to_string()),
            "Keyword '{}' not found as exact token in TextMate grammar pattern",
            keyword
        );
    }
    
    // Optionally check no extra tokens (or allow extras for editor-specific needs)
    // For now, just ensure all required keywords are present
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
    
    // Extract exact tokens from pattern
    let pattern_tokens = extract_tokens_from_pattern(builtin_pattern);
    
    // Check for duplicates
    let mut seen = std::collections::HashSet::new();
    for token in &pattern_tokens {
        assert!(
            seen.insert(token),
            "Duplicate token '{}' found in builtin pattern",
            token
        );
    }
    
    // Check that all builtins appear as exact tokens
    for builtin in BUILTINS {
        assert!(
            pattern_tokens.contains(&builtin.to_string()),
            "Builtin '{}' not found as exact token in TextMate grammar pattern",
            builtin
        );
    }
}

#[test]
fn test_artifact_snapshots() {
    let update_snapshots = std::env::var("SH2C_UPDATE_SNAPSHOTS").is_ok();
    
    // Test VS Code package.json snapshot
    {
        let actual = fs::read_to_string("editors/vscode/package.json")
            .expect("Failed to read package.json");
        let expected_path = "tests/fixtures/editor_package.json.expected";
        
        if update_snapshots {
            fs::write(expected_path, &actual)
                .expect("Failed to write snapshot");
        }
        
        let expected = fs::read_to_string(expected_path)
            .unwrap_or_default();
        
        assert_eq!(
            actual.trim(),
            expected.trim(),
            "VS Code package.json doesn't match snapshot"
        );
    }
    
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
