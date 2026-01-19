//! Regression tests to ensure VS Code extension stays in sync with the language.
//!
//! This test enforces:
//! 1. All required VS Code extension files exist
//! 2. Extension declares .sh2 file extension
//! 3. Grammar contains required keywords/operators (derived from docs/editor_keywords.md)
//! 4. Extension version matches sh2c version

use std::fs;
use std::path::Path;

/// Parsed requirements from docs/editor_keywords.md
struct EditorKeywords {
    keywords: Vec<String>,
    literals: Vec<String>,
    operators: Vec<String>,
}

/// Parse the canonical editor keywords document.
/// Fails with a clear error if the doc structure cannot be parsed.
fn parse_editor_keywords_doc(content: &str) -> EditorKeywords {
    let mut keywords = Vec::new();
    let mut literals = Vec::new();
    let mut operators = Vec::new();
    
    let mut current_section = "";
    let mut in_code_block = false;
    
    for line in content.lines() {
        // Track section headers
        if line.starts_with("## Keywords") {
            current_section = "keywords";
            continue;
        }
        if line.starts_with("## Boolean Literals") {
            current_section = "literals";
            continue;
        }
        if line.starts_with("## Operators") {
            current_section = "operators";
            continue;
        }
        if line.starts_with("## ") {
            current_section = "";
            continue;
        }
        
        // Track code blocks - only when in a recognized section
        if line.starts_with("```") {
            // Only toggle code block state if we're in a section we care about
            if matches!(current_section, "keywords" | "literals" | "operators") {
                in_code_block = !in_code_block;
            }
            continue;
        }
        
        // Extract tokens from code blocks only when in recognized sections
        if in_code_block && !line.trim().is_empty() {
            let tokens: Vec<String> = line.split_whitespace()
                .map(|s| s.to_string())
                .collect();
            
            match current_section {
                "keywords" => keywords.extend(tokens),
                "literals" => literals.extend(tokens),
                "operators" => operators.extend(tokens),
                _ => {} // This case should not be reached due to the guard above
            }
        }
    }
    
    // Validate that we found required sections
    if keywords.is_empty() {
        panic!(
            "Failed to parse docs/editor_keywords.md: no keywords found.\n\
             Expected a fenced code block under '## Keywords ...' section."
        );
    }
    if literals.is_empty() {
        panic!(
            "Failed to parse docs/editor_keywords.md: no boolean literals found.\n\
             Expected a fenced code block under '## Boolean Literals' section."
        );
    }
    if operators.is_empty() {
        panic!(
            "Failed to parse docs/editor_keywords.md: no operators found.\n\
             Expected a fenced code block under '## Operators ...' section."
        );
    }
    
    EditorKeywords { keywords, literals, operators }
}

fn get_repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap()
}

fn load_editor_keywords() -> EditorKeywords {
    let repo = get_repo_root();
    let doc_path = repo.join("docs/editor_keywords.md");
    let content = fs::read_to_string(&doc_path)
        .expect("Could not read docs/editor_keywords.md");
    parse_editor_keywords_doc(&content)
}

#[test]
fn vscode_extension_files_exist() {
    let repo = get_repo_root();
    
    let required_files = [
        "editors/vscode/package.json",
        "editors/vscode/language-configuration.json",
        "editors/vscode/syntaxes/sh2.tmLanguage.json",
    ];
    
    let mut missing = Vec::new();
    for file in &required_files {
        if !repo.join(file).exists() {
            missing.push(*file);
        }
    }
    
    if !missing.is_empty() {
        panic!(
            "VS Code extension is missing {} required file(s):\n  - {}",
            missing.len(),
            missing.join("\n  - ")
        );
    }
}

#[test]
fn vscode_declares_sh2_extension() {
    let repo = get_repo_root();
    let package_json = repo.join("editors/vscode/package.json");
    let content = fs::read_to_string(&package_json)
        .expect("Could not read package.json");
    
    // Check that .sh2 extension is registered
    assert!(
        content.contains("\".sh2\""),
        "package.json must declare .sh2 as a file extension"
    );
    
    // Check language id
    assert!(
        content.contains("\"id\": \"sh2\""),
        "package.json must declare language id 'sh2'"
    );
}

#[test]
fn vscode_grammar_contains_required_keywords() {
    let repo = get_repo_root();
    let grammar = repo.join("editors/vscode/syntaxes/sh2.tmLanguage.json");
    let content = fs::read_to_string(&grammar)
        .expect("Could not read sh2.tmLanguage.json");
    
    let keywords = load_editor_keywords();
    
    let mut missing_keywords = Vec::new();
    for kw in &keywords.keywords {
        // Keywords should appear in the grammar (in a regex like \b(func|let|...)\b
        // or in the builtins section)
        if !content.contains(kw) {
            missing_keywords.push(kw.as_str());
        }
    }
    
    if !missing_keywords.is_empty() {
        panic!(
            "VS Code grammar is missing {} required keyword(s) from docs/editor_keywords.md:\n  - {}\n\n\
            Update editors/vscode/syntaxes/sh2.tmLanguage.json to include these.",
            missing_keywords.len(),
            missing_keywords.join("\n  - ")
        );
    }
}

#[test]
fn vscode_grammar_contains_required_literals() {
    let repo = get_repo_root();
    let grammar = repo.join("editors/vscode/syntaxes/sh2.tmLanguage.json");
    let content = fs::read_to_string(&grammar)
        .expect("Could not read sh2.tmLanguage.json");
    
    let keywords = load_editor_keywords();
    
    let mut missing = Vec::new();
    for lit in &keywords.literals {
        if !content.contains(lit) {
            missing.push(lit.as_str());
        }
    }
    
    if !missing.is_empty() {
        panic!(
            "VS Code grammar is missing {} required literal(s) from docs/editor_keywords.md:\n  - {}",
            missing.len(),
            missing.join("\n  - ")
        );
    }
}

/// parsed grammar rule
#[derive(Debug)]
struct GrammarRule {
    name: String,
    match_pattern: String,
}

/// Parse operators section into rules
fn parse_operator_rules(grammar_content: &str) -> Vec<GrammarRule> {
    let mut rules = Vec::new();
    
    // Find limits of operators section
    let start_idx = grammar_content.find("\"operators\"")
        .expect("grammar missing 'operators' section");
        
    // Find end of section (start of punctuation or end of file)
    let end_idx = grammar_content[start_idx..].find("\"punctuation\"")
        .map(|i| start_idx + i)
        .unwrap_or(grammar_content.len());
        
    let section = &grammar_content[start_idx..end_idx];
    
    // Simple scanner to find "name" and "match" pairs
    // Assumes standard formatting where name comes before match
    let mut current_pos = 0;
    while let Some(name_key) = section[current_pos..].find("\"name\":") {
        let name_idx = current_pos + name_key;
        
        // Extract name value
        let val_start = section[name_idx..].find(':')
            .map(|i| name_idx + i + 1).unwrap();
        
        // Skip whitespace to quote
        let quote_start = section[val_start..].find('"')
            .map(|i| val_start + i).unwrap();
            
        let quote_end = section[quote_start+1..].find('"')
            .map(|i| quote_start + 1 + i).unwrap();
            
        let name_val = &section[quote_start+1..quote_end];
        
        // Find corresponding match
        // Limit search to next rule or end
        let search_limit = section[quote_end..].find("\"name\":")
            .map(|i| quote_end + i)
            .unwrap_or(section.len());
            
        let match_key = match section[quote_end..search_limit].find("\"match\":") {
            Some(i) => quote_end + i,
            None => {
                // Rule might not have a match (e.g. include), skip
                current_pos = quote_end;
                continue;
            }
        };
        
        // Extract match value
        let m_val_start = section[match_key..].find(':')
            .map(|i| match_key + i + 1).unwrap();
            
        let m_quote_start = section[m_val_start..].find('"')
            .map(|i| m_val_start + i).unwrap();
            
        let m_quote_end = section[m_quote_start+1..].find('"')
            .map(|i| m_quote_start + 1 + i).unwrap();
            
        let match_val = &section[m_quote_start+1..m_quote_end];
        
        rules.push(GrammarRule {
            name: name_val.to_string(),
            match_pattern: match_val.to_string(),
        });
        
        current_pos = m_quote_end;
    }
    
    if rules.is_empty() {
        panic!("could not parse operators.patterns rules");
    }
    
    rules
}

/// Check if a regex pattern matches a specific token
/// Handles JSON unescaping and regex alternations robustly
fn check_pattern_matches_token(raw_pattern: &str, token: &str) -> bool {
    // 1. Unescape JSON string to get the actual regex pattern
    // Wrap in quotes to make it a valid JSON string for parsing
    let json_str = format!("\"{}\"", raw_pattern);
    let regex_pattern: String = serde_json::from_str(&json_str)
        .unwrap_or_else(|_| raw_pattern.to_string());

    // 2. Exact match check
    if regex_pattern == token { return true; }
    
    // 3. Handle alternations (a|b|c)
    // Strip parens
    let inner = regex_pattern.trim_start_matches('(').trim_end_matches(')');
    
    // Split by | but respect escaped pipes \|
    let parts = split_regex_alternation(inner);
    
    for part in parts {
        // Unescape regex special chars in the part
        // e.g. \| becomes |
        let unescaped = unescape_regex(&part);
        if unescaped == token {
            return true;
        }
        if part == token { return true; }
    }
    
    false
}

fn split_regex_alternation(pattern: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut chars = pattern.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == '\\' {
            current.push(c);
            if let Some(&next) = chars.peek() {
                // If next is pipe, it's escaped, consume it and continue
                // Otherwise it's just a backslash (maybe escaping something else)
                if next == '|' {
                    chars.next();
                    current.push('|');
                }
            }
        } else if c == '|' {
            parts.push(current);
            current = String::new();
        } else {
            current.push(c);
        }
    }
    parts.push(current);
    parts
}

fn unescape_regex(s: &str) -> String {
    s.replace("\\|", "|")
     .replace("\\!", "!")
     .replace("\\&", "&")
     .replace("\\+", "+")
     .replace("\\*", "*")
     .replace("\\\\", "\\")
}

/// Check if an operator is properly defined in the operators section of the grammar.
/// 
/// This function relies on parsing the operators section into rules and verifying
/// the required operators exist with correct rule names and match patterns.
fn is_operator_defined(grammar_content: &str, op: &str) -> bool {
    let rules = parse_operator_rules(grammar_content);
    
    match op {
        // Dedicated rules
        "!" => rules.iter().any(|r| 
            r.name == "keyword.operator.not.sh2" && check_pattern_matches_token(&r.match_pattern, "!")
        ),
        
        "&" => rules.iter().any(|r| 
            r.name == "keyword.operator.concat.sh2" && check_pattern_matches_token(&r.match_pattern, "&")
        ),
        
        "|" => rules.iter().any(|r| 
            r.name == "keyword.operator.pipe.sh2" && check_pattern_matches_token(&r.match_pattern, "|")
        ),
        
        "|>" => rules.iter().any(|r| 
            r.name == "keyword.operator.pipe-block.sh2" && check_pattern_matches_token(&r.match_pattern, "|>")
        ),
        
        "=>" => rules.iter().any(|r| 
            r.name == "keyword.operator.arrow.sh2" && check_pattern_matches_token(&r.match_pattern, "=>")
        ),
        
        // Logical operators
        "&&" | "||" => {
            rules.iter().any(|r| {
                r.name == "keyword.operator.logical.sh2" && 
                check_pattern_matches_token(&r.match_pattern, "&&") &&
                check_pattern_matches_token(&r.match_pattern, "||")
            })
        },
        
        // Comparison operators
        "<" | ">" | "<=" | ">=" | "==" | "!=" => {
            rules.iter().any(|r| {
                r.name == "keyword.operator.comparison.sh2" &&
                check_pattern_matches_token(&r.match_pattern, op)
            })
        },
        
        _ => false // Not supporting unknown operators in this strict check
    }
}

#[test]
fn test_json_escaped_match_coverage() {
    // raw pattern: (&&|\|\|) -> matches && or ||
    // In JSON: "match": "(&&|\\|\\|)"
    let json_pattern = "(&&|\\\\|\\\\|)"; 
    assert!(check_pattern_matches_token(json_pattern, "&&"));
    assert!(check_pattern_matches_token(json_pattern, "||"));
    
    // raw regex: \| -> matches |
    // In JSON: "match": "\\|"
    let json_pipe = "\\\\|";
    assert!(check_pattern_matches_token(json_pipe, "|"));
    
    // raw regex: \|> -> matches |>
    // In JSON: "match": "\\|>"
    let json_pipe_block = "\\\\|>";
    assert!(check_pattern_matches_token(json_pipe_block, "|>"));
}



#[test]
fn vscode_grammar_contains_required_operators() {
    let repo = get_repo_root();
    let grammar = repo.join("editors/vscode/syntaxes/sh2.tmLanguage.json");
    let content = fs::read_to_string(&grammar)
        .expect("Could not read sh2.tmLanguage.json");
    
    let keywords = load_editor_keywords();
    
    let mut missing = Vec::new();
    for op in &keywords.operators {
        if !is_operator_defined(&content, op) {
            missing.push(op.as_str());
        }
    }
    
    if !missing.is_empty() {
        panic!(
            "VS Code grammar is missing {} required operator(s) from docs/editor_keywords.md:\n  - {}\n\n\
            Update editors/vscode/syntaxes/sh2.tmLanguage.json to include these.",
            missing.len(),
            missing.join("\n  - ")
        );
    }
}

#[test]
fn vscode_uses_hash_comments() {
    let repo = get_repo_root();
    let lang_config = repo.join("editors/vscode/language-configuration.json");
    let content = fs::read_to_string(&lang_config)
        .expect("Could not read language-configuration.json");
    
    // sh2 uses # for comments, not //
    assert!(
        content.contains("\"lineComment\": \"#\""),
        "language-configuration.json must use '#' for lineComment, not '//'"
    );
}

#[test]
fn vscode_version_matches_sh2c() {
    let repo = get_repo_root();
    
    // Read sh2c version from Cargo.toml
    let cargo_toml = repo.join("sh2c/Cargo.toml");
    let cargo_content = fs::read_to_string(&cargo_toml)
        .expect("Could not read sh2c/Cargo.toml");
    
    // Extract version line: version = "X.Y.Z"
    let sh2c_version = cargo_content
        .lines()
        .find(|line| line.starts_with("version = "))
        .and_then(|line| {
            line.split('"')
                .nth(1)
                .map(|v| v.to_string())
        })
        .expect("Could not find version in sh2c/Cargo.toml");
    
    // Read VS Code extension version from package.json
    let package_json = repo.join("editors/vscode/package.json");
    let package_content = fs::read_to_string(&package_json)
        .expect("Could not read package.json");
    
    // Extract version: "version": "X.Y.Z"
    let vscode_version = package_content
        .lines()
        .find(|line| line.contains("\"version\":"))
        .and_then(|line| {
            line.split('"')
                .nth(3)
                .map(|v| v.to_string())
        })
        .expect("Could not find version in package.json");
    
    assert_eq!(
        sh2c_version, vscode_version,
        "VS Code extension version ({}) must match sh2c version ({})",
        vscode_version, sh2c_version
    );
}

// ============================================================================
// Parser unit tests
// ============================================================================

#[test]
fn test_parse_editor_keywords_valid() {
    let doc = r#"
# sh2 Editor Keywords Reference

## Keywords (Control Flow)

```
func let if else
```

## Boolean Literals

```
true false
```

## Operators / Syntax Tokens

```text
=> |> | &
```

## Comment Marker
"#;
    
    let parsed = parse_editor_keywords_doc(doc);
    
    assert_eq!(parsed.keywords, vec!["func", "let", "if", "else"]);
    assert_eq!(parsed.literals, vec!["true", "false"]);
    assert_eq!(parsed.operators, vec!["=>", "|>", "|", "&"]);
}

#[test]
#[should_panic(expected = "no keywords found")]
fn test_parse_editor_keywords_missing_keywords() {
    let doc = r#"
# sh2 Editor Keywords Reference

## Boolean Literals

```
true false
```

## Operators

```
=> |>
```
"#;
    
    parse_editor_keywords_doc(doc);
}

#[test]
#[should_panic(expected = "no boolean literals found")]
fn test_parse_editor_keywords_missing_literals() {
    let doc = r#"
## Keywords

```
func let
```

## Operators

```
=> |>
```
"#;
    
    parse_editor_keywords_doc(doc);
}

#[test]
#[should_panic(expected = "no operators found")]
fn test_parse_editor_keywords_missing_operators() {
    let doc = r#"
## Keywords

```
func let
```

## Boolean Literals

```
true false
```
"#;
    
    parse_editor_keywords_doc(doc);
}

// ============================================================================
// is_operator_defined() unit tests
// ============================================================================

#[test]
fn test_operator_not_requires_dedicated_rule() {
    // Content has != but not standalone ! operator rule
    let content_without_not = r#"
        "operators": {
            "patterns": [
                {
                    "name": "keyword.operator.comparison.sh2",
                    "match": "(==|!=)"
                }
            ]
        }
        "punctuation": {
    "#;
    
    // ! should NOT be satisfied just because != is present
    assert!(!is_operator_defined(content_without_not, "!"),
        "! should not be satisfied by != alone");
    
    // Content with dedicated not operator rule
    let content_with_not = r#"
        "operators": {
            "patterns": [
                {
                    "name": "keyword.operator.not.sh2",
                    "match": "!"
                }
            ]
        }
        "punctuation": {
    "#;
    
    assert!(is_operator_defined(content_with_not, "!"),
        "! should be satisfied by dedicated operator.not rule");
}

#[test]
fn test_operator_concat_requires_dedicated_rule() {
    // Content has && but not standalone & operator rule
    let content_without_concat = r#"
        "operators": {
            "patterns": [
                {
                    "name": "keyword.operator.logical.sh2",
                    "match": "(&&|\\|\\|)"
                }
            ]
        }
        "punctuation": {
    "#;
    
    // & should NOT be satisfied just because && is present
    assert!(!is_operator_defined(content_without_concat, "&"),
        "& should not be satisfied by && alone");
    
    // Content with dedicated concat operator rule
    let content_with_concat = r#"
        "operators": {
            "patterns": [
                {
                    "name": "keyword.operator.concat.sh2",
                    "match": "&"
                }
            ]
        }
        "punctuation": {
    "#;
    
    assert!(is_operator_defined(content_with_concat, "&"),
        "& should be satisfied by dedicated operator.concat rule");
}

#[test]
fn test_operator_pipe_requires_dedicated_rule() {
    // Content has |> and || but not standalone | operator rule
    let content_without_pipe = r#"
        "operators": {
            "patterns": [
                {
                    "name": "keyword.operator.pipe-block.sh2",
                    "match": "\\|>"
                },
                {
                    "name": "keyword.operator.logical.sh2",
                    "match": "(&&|\\|\\|)"
                }
            ]
        }
        "punctuation": {
    "#;
    
    // | should NOT be satisfied by |> or ||
    assert!(!is_operator_defined(content_without_pipe, "|"),
        "| should not be satisfied by |> or || alone");
    
    // Content with dedicated pipe operator rule
    let content_with_pipe = r#"
        "operators": {
            "patterns": [
                {
                    "name": "keyword.operator.pipe.sh2",
                    "match": "\\|"
                }
            ]
        }
        "punctuation": {
    "#;
    
    assert!(is_operator_defined(content_with_pipe, "|"),
        "| should be satisfied by dedicated keyword.operator.pipe.sh2 rule");
}

#[test]
fn test_comparison_operators_check_comparison_rule() {
    let content_with_comparison = r#"
        "operators": {
            "patterns": [
                {
                    "name": "keyword.operator.comparison.sh2",
                    "match": "(==|!=|<=|>=|<|>)"
                }
            ]
        }
        "punctuation": {
    "#;
    
    // All comparison operators should be found
    assert!(is_operator_defined(content_with_comparison, "<"));
    assert!(is_operator_defined(content_with_comparison, ">"));
    assert!(is_operator_defined(content_with_comparison, "<="));
    assert!(is_operator_defined(content_with_comparison, ">="));
    assert!(is_operator_defined(content_with_comparison, "=="));
    assert!(is_operator_defined(content_with_comparison, "!="));
}

#[test]
fn test_logical_operators_check_logical_rule() {
    let content_with_logical = r#"
        "operators": {
            "patterns": [
                {
                    "name": "keyword.operator.logical.sh2",
                    "match": "(&&|\\|\\|)"
                }
            ]
        }
        "punctuation": {
    "#;
    
    assert!(is_operator_defined(content_with_logical, "&&"));
    assert!(is_operator_defined(content_with_logical, "||"));
}

#[test]
fn test_parse_comment_block_does_not_pollute_operators() {
    // Doc has a comment marker code block that should NOT be parsed as operators
    let doc = r#"
## Keywords

```
func let
```

## Boolean Literals

```
true false
```

## Operators / Syntax Tokens

```text
=> |>
```

## Comment Marker

```
# single-line comment
```

## Maintenance

Some text here.
"#;
    
    let parsed = parse_editor_keywords_doc(doc);
    
    // Operators should only contain => and |>, not # from comment marker section
    assert_eq!(parsed.operators, vec!["=>", "|>"]);
    assert!(!parsed.operators.contains(&"#".to_string()),
        "Comment marker '#' should not be captured as an operator");
}

#[test]
fn test_comparison_operator_does_not_false_positive_on_prefix() {
    // Grammar content where comparison match only has <= and >= (not < and >)
    let content_without_lt_gt = r#"
        "operators": {
            "patterns": [
                {
                    "name": "keyword.operator.comparison.sh2",
                    "match": "(==|!=|<=|>=)"
                }
            ]
        }
        "punctuation": {
    "#;
    
    // < and > should NOT be satisfied when only <= and >= exist
    assert!(!is_operator_defined(content_without_lt_gt, "<"),
        "< should not be satisfied when only <= exists in comparison pattern");
    assert!(!is_operator_defined(content_without_lt_gt, ">"),
        "> should not be satisfied when only >= exists in comparison pattern");
    
    // But <= and >= should still be found
    assert!(is_operator_defined(content_without_lt_gt, "<="));
    assert!(is_operator_defined(content_without_lt_gt, ">="));
    assert!(is_operator_defined(content_without_lt_gt, "=="));
    assert!(is_operator_defined(content_without_lt_gt, "!="));
    
    // Also test the reverse: only < and > without <= and >=
    let content_with_only_lt_gt = r#"
        "operators": {
            "patterns": [
                {
                    "name": "keyword.operator.comparison.sh2",
                    "match": "(==|!=|<|>)"
                }
            ]
        }
        "punctuation": {
    "#;
    
    // < and > should be found
    assert!(is_operator_defined(content_with_only_lt_gt, "<"));
    assert!(is_operator_defined(content_with_only_lt_gt, ">"));
    // But <= and >= should NOT be found
    assert!(!is_operator_defined(content_with_only_lt_gt, "<="),
        "<= should not be satisfied when only < exists");
    assert!(!is_operator_defined(content_with_only_lt_gt, ">="),
        ">= should not be satisfied when only > exists");
}
