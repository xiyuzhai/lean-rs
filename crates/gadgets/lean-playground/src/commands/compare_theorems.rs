use lean_parser::Parser as LeanParser;
use lean_syn_expr::{Syntax, SyntaxKind};
use std::collections::HashMap;
use std::path::PathBuf;

pub fn compare_theorem_files(dir: &PathBuf) {
    // Find all .lean files and match them with .Decl.lean files
    let entries = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("Failed to read directory {}: {}", dir.display(), e));

    let mut pairs: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut lean_files: Vec<PathBuf> = Vec::new();
    let mut decl_files: HashMap<String, PathBuf> = HashMap::new();

    // Collect all files
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.ends_with(".Decl.lean") {
                // Extract base name (XXX from XXX.Decl.lean)
                let base_name = name.strip_suffix(".Decl.lean").unwrap();
                decl_files.insert(base_name.to_string(), path);
            } else if name.ends_with(".lean") {
                lean_files.push(path);
            }
        }
    }

    // Match .lean files with .Decl.lean files
    for lean_file in lean_files {
        if let Some(name) = lean_file.file_name().and_then(|n| n.to_str()) {
            if let Some(base_name) = name.strip_suffix(".lean") {
                if let Some(decl_file) = decl_files.get(base_name) {
                    pairs.push((lean_file.clone(), decl_file.clone()));
                }
            }
        }
    }

    if pairs.is_empty() {
        println!("No matching pairs of .lean and .Decl.lean files found");
        return;
    }

    println!("Found {} pair(s) to compare", pairs.len());
    println!();

    let mut all_match = true;

    for (lean_file, decl_file) in pairs {
        println!("Comparing:");
        println!("  {}", lean_file.display());
        println!("  {}", decl_file.display());
        println!();

        // Parse both files
        let lean_theorems = extract_prop_theorems(&lean_file);
        let decl_theorems = extract_prop_theorems(&decl_file);

        // Compare theorems
        let mut has_differences = false;

        // Check for theorems in lean_file but not in decl_file
        for (name, ast) in &lean_theorems {
            match decl_theorems.get(name) {
                Some(decl_ast) => {
                    if !asts_equal(ast, decl_ast) {
                        println!("  ✗ Theorem '{}' head differs", name);
                        has_differences = true;
                    }
                }
                None => {
                    println!("  ✗ Theorem '{}' missing in {}", name, decl_file.display());
                    has_differences = true;
                }
            }
        }

        // Check for theorems in decl_file but not in lean_file
        for name in decl_theorems.keys() {
            if !lean_theorems.contains_key(name) {
                println!("  ✗ Theorem '{}' missing in {}", name, lean_file.display());
                has_differences = true;
            }
        }

        if has_differences {
            all_match = false;
        } else {
            println!("  ✓ All prop: theorems match!");
        }
        println!();
    }

    if all_match {
        println!("✓ All pairs match!");
    } else {
        println!("✗ Some pairs have differences");
    }
}

fn extract_prop_theorems(file_path: &PathBuf) -> HashMap<String, TheoremHead> {
    // Read the file
    let content = std::fs::read_to_string(file_path)
        .unwrap_or_else(|e| panic!("Failed to read file {}: {}", file_path.display(), e));

    // Parse the file
    let mut parser = LeanParser::new(&content);
    let syntax_tree = parser
        .module()
        .unwrap_or_else(|e| panic!("Failed to parse file {}: {:?}", file_path.display(), e));

    // Extract prop: theorems
    let mut theorems = HashMap::new();
    collect_prop_theorems(&syntax_tree, &mut theorems);

    theorems
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TheoremHead {
    name: String,
    params: Vec<SyntaxRepr>,
    type_expr: SyntaxRepr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SyntaxRepr {
    Node {
        kind: SyntaxKind,
        children: Vec<SyntaxRepr>,
    },
    Atom(String),
    Missing,
}

fn collect_prop_theorems(syntax: &Syntax, theorems: &mut HashMap<String, TheoremHead>) {
    match syntax {
        Syntax::Node(node) => {
            // If this is a theorem node, check if name contains "prop:"
            if node.kind == SyntaxKind::Theorem {
                if let Some(theorem_head) = extract_theorem_head(node.children.as_slice()) {
                    // Check if name contains "prop:" (handles guillemets «prop:...»)
                    if theorem_head.name.contains("prop:") {
                        theorems.insert(theorem_head.name.clone(), theorem_head);
                    }
                }
            }
            // Recursively process all children
            for child in &node.children {
                collect_prop_theorems(child, theorems);
            }
        }
        Syntax::Atom(_) | Syntax::Missing => {
            // Atoms and missing nodes don't have children
        }
    }
}

fn extract_theorem_head(children: &[Syntax]) -> Option<TheoremHead> {
    // Theorem structure: [name, params..., type, proof]
    // We need to extract: name, params, and type (excluding proof which is last)

    if children.is_empty() {
        return None;
    }

    // First child is the name
    let name = match &children[0] {
        Syntax::Atom(atom) => atom.value.to_string(),
        _ => return None,
    };

    // Last child is the proof, second to last is the type
    if children.len() < 2 {
        return None;
    }

    let type_expr = syntax_to_repr(&children[children.len() - 2]);

    // Everything between name and type are parameters
    let mut params = Vec::new();
    for i in 1..children.len() - 2 {
        params.push(syntax_to_repr(&children[i]));
    }

    Some(TheoremHead {
        name,
        params,
        type_expr,
    })
}

fn syntax_to_repr(syntax: &Syntax) -> SyntaxRepr {
    match syntax {
        Syntax::Node(node) => {
            let children = node.children.iter().map(syntax_to_repr).collect();
            SyntaxRepr::Node {
                kind: node.kind,
                children,
            }
        }
        Syntax::Atom(atom) => SyntaxRepr::Atom(atom.value.to_string()),
        Syntax::Missing => SyntaxRepr::Missing,
    }
}

fn asts_equal(a: &TheoremHead, b: &TheoremHead) -> bool {
    a == b
}
