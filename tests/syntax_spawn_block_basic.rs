mod common;
use common::*;
use sh2c::ast::{Stmt};

#[test]
fn parse_spawn_block_basic() {
    let program = parse_fixture("spawn_block_basic");
    let func = &program.functions[0];
    
    // Check Spawn { stmt: Group { ... } }
    if let Stmt::Spawn { stmt } = &func.body[0] {
        if let Stmt::Group { body } = &**stmt {
             assert_eq!(body.len(), 1);
             if let Stmt::Run(..) = &body[0] {
                 // OK
             } else {
                 panic!("Expected Run inside Group");
             }
        } else {
             panic!("Expected Stmt::Group inside Spawn");
        }
    } else {
        panic!("Expected Stmt::Spawn");
    }
}

#[test]
fn codegen_spawn_block_basic() {
    assert_codegen_matches_snapshot("spawn_block_basic");
}

#[test]
fn exec_spawn_block_basic() {
    assert_exec_matches_fixture("spawn_block_basic");
}
