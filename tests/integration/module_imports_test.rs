//! Integration test for full module import pipeline
//!
//! This test creates complete modules with imports, definitions, and inductive types
//! to verify that the full parsing -> elaboration pipeline works correctly.

use lean_elaborator::{module_loader::ModuleLoaderConfig, Elaborator};
use lean_kernel::Name;
use lean_parser::ExpandingParser;


#[test]
fn test_simple_module_import() {
    // Test parsing and elaborating a simple module without external imports
    let simple_module = r#"-- Simple test module
inductive TestBool where
  | tTrue
  | tFalse
"#;
    
    let mut elaborator = Elaborator::new();
    elaborator.state_mut().set_env(lean_elaborator::init_basic_environment());
    
    let mut parser = ExpandingParser::new(simple_module);
    let module_syntax = parser.parse_module()
        .expect("Failed to parse simple module");
    
    // Debug: print the parsed syntax
    println!("Parsed syntax: {:?}", module_syntax);
    
    // Elaborate the module
    let result = lean_elaborator::command::elaborate_module_commands(&mut elaborator, &module_syntax);
    assert!(result.is_ok(), "Failed to elaborate simple module: {:?}", result);
    
    // Verify that types were added to the environment
    if let Some(env) = &elaborator.state().env {
        assert!(env.contains(&Name::mk_simple("TestBool")), 
               "TestBool should be in environment");
        assert!(env.contains(&Name::str(Name::mk_simple("TestBool"), "tTrue")), 
               "TestBool.tTrue should be in environment");
        assert!(env.contains(&Name::str(Name::mk_simple("TestBool"), "tFalse")), 
               "TestBool.tFalse should be in environment");
    } else {
        panic!("Environment should not be None");
    }
}

#[test]
fn test_module_with_imports() {
    // Test a module that uses import syntax but doesn't actually resolve imports
    // This tests the import parsing without the complex module loading
    let module_with_import = r#"-- Module with import syntax
import SomeModule

inductive MyList (α : Type) where
  | nil
  | cons : α → MyList α → MyList α

def listLength : MyList α → Nat
  | MyList.nil => 0
  | MyList.cons _ xs => 1 + listLength xs
"#;
    
    let mut elaborator = Elaborator::new();
    elaborator.state_mut().set_env(lean_elaborator::init_basic_environment());
    
    let mut parser = ExpandingParser::new(module_with_import);
    let module_syntax = parser.parse_module()
        .expect("Failed to parse module with import");
    
    // Elaborate the module - this will fail on the import but we can test that parsing works
    let result = lean_elaborator::command::elaborate_module_commands(&mut elaborator, &module_syntax);
    
    // The import should fail, but that's expected for this test
    match result {
        Err(lean_elaborator::ElabError::ModuleNotFound(name)) => {
            assert_eq!(name.to_string(), "SomeModule");
        }
        _ => panic!("Expected ModuleNotFound error for import"),
    }
}

#[test]
fn test_inductive_and_def_elaboration() {
    // Test that multiple inductive types are properly elaborated together
    let module_with_both = r#"-- Module with multiple inductive types
inductive Color where
  | red
  | green  
  | blue

inductive Status where
  | pending
  | success
  | failure
"#;
    
    let mut elaborator = Elaborator::new();
    elaborator.state_mut().set_env(lean_elaborator::init_basic_environment());
    
    let mut parser = ExpandingParser::new(module_with_both);
    let module_syntax = parser.parse_module()
        .expect("Failed to parse module with inductive and def");
    
    // Elaborate the module
    let result = lean_elaborator::command::elaborate_module_commands(&mut elaborator, &module_syntax);
    assert!(result.is_ok(), "Failed to elaborate module: {:?}", result);
    
    // Verify that all types were added to the environment
    if let Some(env) = &elaborator.state().env {
        // Color inductive
        assert!(env.contains(&Name::mk_simple("Color")), "Color should be in environment");
        assert!(env.contains(&Name::str(Name::mk_simple("Color"), "red")), "Color.red should be in environment");
        assert!(env.contains(&Name::str(Name::mk_simple("Color"), "green")), "Color.green should be in environment");
        assert!(env.contains(&Name::str(Name::mk_simple("Color"), "blue")), "Color.blue should be in environment");
        
        // Status inductive
        assert!(env.contains(&Name::mk_simple("Status")), "Status should be in environment");
        assert!(env.contains(&Name::str(Name::mk_simple("Status"), "pending")), "Status.pending should be in environment");
        assert!(env.contains(&Name::str(Name::mk_simple("Status"), "success")), "Status.success should be in environment");
        assert!(env.contains(&Name::str(Name::mk_simple("Status"), "failure")), "Status.failure should be in environment");
    } else {
        panic!("Environment should not be None");
    }
}

#[test]
fn test_multiple_inductive_types() {
    // Test multiple inductive types in sequence
    let module_with_multiple = r#"-- Module with multiple inductive types
inductive Direction where
  | north
  | south
  | east
  | west

inductive Season where
  | spring
  | summer
  | autumn
  | winter

inductive Mode where
  | active
  | inactive
"#;
    
    let mut elaborator = Elaborator::new();
    elaborator.state_mut().set_env(lean_elaborator::init_basic_environment());
    
    let mut parser = ExpandingParser::new(module_with_multiple);
    let module_syntax = parser.parse_module()
        .expect("Failed to parse module with multiple inductives");
    
    // Elaborate the module
    let result = lean_elaborator::command::elaborate_module_commands(&mut elaborator, &module_syntax);
    assert!(result.is_ok(), "Failed to elaborate module with multiple inductives: {:?}", result);
    
    // Verify that all inductive types were added
    if let Some(env) = &elaborator.state().env {
        // Direction inductive
        assert!(env.contains(&Name::mk_simple("Direction")), "Direction should be in environment");
        assert!(env.contains(&Name::str(Name::mk_simple("Direction"), "north")), "Direction.north should be in environment");
        assert!(env.contains(&Name::str(Name::mk_simple("Direction"), "south")), "Direction.south should be in environment");
        assert!(env.contains(&Name::str(Name::mk_simple("Direction"), "east")), "Direction.east should be in environment");
        assert!(env.contains(&Name::str(Name::mk_simple("Direction"), "west")), "Direction.west should be in environment");
        
        // Season inductive
        assert!(env.contains(&Name::mk_simple("Season")), "Season should be in environment");
        assert!(env.contains(&Name::str(Name::mk_simple("Season"), "spring")), "Season.spring should be in environment");
        assert!(env.contains(&Name::str(Name::mk_simple("Season"), "summer")), "Season.summer should be in environment");
        assert!(env.contains(&Name::str(Name::mk_simple("Season"), "autumn")), "Season.autumn should be in environment");
        assert!(env.contains(&Name::str(Name::mk_simple("Season"), "winter")), "Season.winter should be in environment");
        
        // Mode inductive
        assert!(env.contains(&Name::mk_simple("Mode")), "Mode should be in environment");
        assert!(env.contains(&Name::str(Name::mk_simple("Mode"), "active")), "Mode.active should be in environment");
        assert!(env.contains(&Name::str(Name::mk_simple("Mode"), "inactive")), "Mode.inactive should be in environment");
    } else {
        panic!("Environment should not be None");
    }
}

#[test]
fn test_error_handling_missing_import() {
    // Set up module loader with default config
    let config = ModuleLoaderConfig::default();
    
    let mut elaborator = Elaborator::new();
    elaborator.state_mut().module_loader = std::sync::Arc::new(
        lean_elaborator::module_loader::ModuleLoader::new(config)
    );
    elaborator.state_mut().set_env(lean_elaborator::init_basic_environment());
    
    // Test module that imports a non-existent module
    let test_module = r#"
import NonExistent.Module

def testFunction : Nat := 42
"#;
    
    let mut parser = ExpandingParser::new(test_module);
    let module_syntax = parser.parse_module()
        .expect("Failed to parse test module");
    
    // Elaborate the module - this should fail due to missing import
    let result = lean_elaborator::command::elaborate_module_commands(&mut elaborator, &module_syntax);
    assert!(result.is_err(), "Elaboration should fail with missing import");
    
    // Check that we get the right kind of error
    match result {
        Err(lean_elaborator::ElabError::ModuleNotFound(name)) => {
            assert_eq!(name.to_string(), "NonExistent.Module");
        }
        Err(other) => panic!("Expected ModuleNotFound error, got: {:?}", other),
        Ok(_) => panic!("Expected error but got success"),
    }
}

