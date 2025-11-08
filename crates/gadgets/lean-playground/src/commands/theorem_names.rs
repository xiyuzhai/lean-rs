use lean_parser::Parser as LeanParser;
use lean_syn_expr::{Syntax, SyntaxKind};
use std::path::PathBuf;

pub fn extract_and_print_theorem_names(file_path: &PathBuf) {
    // Read the file
    let content = std::fs::read_to_string(file_path)
        .unwrap_or_else(|e| panic!("Failed to read file {}: {}", file_path.display(), e));

    // Parse the file
    let mut parser = LeanParser::new(&content);
    let syntax_tree = parser
        .module()
        .unwrap_or_else(|e| panic!("Failed to parse file: {:?}", e));

    // Extract theorem names
    let mut theorem_names = Vec::new();
    collect_theorem_names(&syntax_tree, &mut theorem_names);

    // Print each theorem name on a new line
    for name in theorem_names {
        println!("{}", name);
    }
}

fn collect_theorem_names(syntax: &Syntax, names: &mut Vec<String>) {
    match syntax {
        Syntax::Node(node) => {
            // If this is a theorem node, extract its name
            if node.kind == SyntaxKind::Theorem {
                if let Some(name) = extract_theorem_name(node.children.as_slice()) {
                    names.push(name);
                }
            }
            // Recursively process all children
            for child in &node.children {
                collect_theorem_names(child, names);
            }
        }
        Syntax::Atom(_) | Syntax::Missing => {
            // Atoms and missing nodes don't have children
        }
    }
}

fn extract_theorem_name(children: &[Syntax]) -> Option<String> {
    // The first child of a theorem node should be the identifier (name)
    children.first().and_then(|child| match child {
        Syntax::Atom(atom) => Some(atom.value.to_string()),
        _ => None,
    })
}
