mod common;

#[test]
fn test_arg_zero_lowers_to_dynamic() {
    use sh2c::ast;
    use sh2c::lexer;
    use sh2c::lower;
    use sh2c::parser;
    use sh2c::span::SourceMap;
    
    let src = r#"
        func main() {
            let x = arg(0)
        }
    "#;
    
    let sm = SourceMap::new(src.to_string());
    let tokens = lexer::lex(&sm, src).expect("lex failed");
    let program = parser::parse(&tokens, &sm, "test").expect("parse failed");
    
    let opts = lower::LowerOptions {
        include_diagnostics: false,
        diag_base_dir: None,
    };
    
    let mut prog_with_maps = program;
    prog_with_maps.source_maps.insert("test".to_string(), sm);
    
    let ir = lower::lower_with_options(prog_with_maps, &opts).expect("lower failed");
    
    // Find the let statement in main function
    let main_fn = ir.iter().find(|f| f.name == "main").expect("main not found");
    
    // First command should be Assign for "x"
    if let sh2c::ir::Cmd::Assign(name, val, _) = &main_fn.commands[0] {
        assert_eq!(name, "x");
        
        // arg(0) should lower to ArgDynamic, NOT Arg(0)
        match val {
            sh2c::ir::Val::ArgDynamic(_) => {
                // Success - arg(0) correctly uses dynamic path
            }
            sh2c::ir::Val::Arg(n) => {
                panic!("arg(0) incorrectly lowered to Val::Arg({}) instead of ArgDynamic", n);
            }
            _ => {
                panic!("Unexpected value type for arg(0): {:?}", val);
            }
        }
    } else {
        panic!("Expected Assign command");
    }
}

#[test]
fn test_arg_one_lowers_to_fast_path() {
    use sh2c::ast;
    use sh2c::lexer;
    use sh2c::lower;
    use sh2c::parser;
    use sh2c::span::SourceMap;
    
    let src = r#"
        func main() {
            let x = arg(1)
        }
    "#;
    
    let sm = SourceMap::new(src.to_string());
    let tokens = lexer::lex(&sm, src).expect("lex failed");
    let program = parser::parse(&tokens, &sm, "test").expect("parse failed");
    
    let opts = lower::LowerOptions {
        include_diagnostics: false,
        diag_base_dir: None,
    };
    
    let mut prog_with_maps = program;
    prog_with_maps.source_maps.insert("test".to_string(), sm);
    
    let ir = lower::lower_with_options(prog_with_maps, &opts).expect("lower failed");
    
    // Find the let statement in main function
    let main_fn = ir.iter().find(|f| f.name == "main").expect("main not found");
    
    // First command should be Assign for "x"
    if let sh2c::ir::Cmd::Assign(name, val, _) = &main_fn.commands[0] {
        assert_eq!(name, "x");
        
        // arg(1) should use fast path Val::Arg(1)
        match val {
            sh2c::ir::Val::Arg(1) => {
                // Success - arg(1) correctly uses fast path
            }
            sh2c::ir::Val::ArgDynamic(_) => {
                panic!("arg(1) should use fast path Val::Arg(1), not ArgDynamic");
            }
            _ => {
                panic!("Unexpected value type for arg(1): {:?}", val);
            }
        }
    } else {
        panic!("Expected Assign command");
    }
}
