use tree_sitter::Language;

/// Detect the tree-sitter Language for a file based on its extension.
pub fn language_for_extension(ext: &str) -> Option<Language> {
    match ext {
        "rs" => Some(tree_sitter_rust::LANGUAGE.into()),
        "py" => Some(tree_sitter_python::LANGUAGE.into()),
        "js" | "jsx" | "mjs" | "cjs" => Some(tree_sitter_javascript::LANGUAGE.into()),
        "ts" => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        "tsx" => Some(tree_sitter_typescript::LANGUAGE_TSX.into()),
        "go" => Some(tree_sitter_go::LANGUAGE.into()),
        "c" | "h" => Some(tree_sitter_c::LANGUAGE.into()),
        "cpp" | "cc" | "cxx" | "hpp" | "hh" => Some(tree_sitter_cpp::LANGUAGE.into()),
        _ => None,
    }
}

/// Returns the tree-sitter query string for function definitions in the given language.
/// The query captures the function name under `@name`.
pub fn function_def_query(ext: &str) -> Option<&'static str> {
    match ext {
        "rs" => Some(r#"(function_item name: (identifier) @name)"#),
        "py" => Some(r#"(function_definition name: (identifier) @name)"#),
        "js" | "jsx" | "mjs" | "cjs" => Some(
            r#"(function_declaration name: (identifier) @name)
               (method_definition name: (property_identifier) @name)
               (arrow_function)"#,
        ),
        "ts" | "tsx" => Some(
            r#"(function_declaration name: (identifier) @name)
               (method_definition name: (property_identifier) @name)"#,
        ),
        "go" => Some(r#"(function_declaration name: (identifier) @name)"#),
        "c" | "h" | "cpp" | "cc" | "cxx" | "hpp" | "hh" => {
            Some(r#"(function_definition declarator: (function_declarator declarator: (identifier) @name))"#)
        }
        _ => None,
    }
}

/// Returns the tree-sitter query string for call expressions (callee name under `@callee`).
pub fn call_query(ext: &str) -> Option<&'static str> {
    match ext {
        "rs" => Some(
            r#"(call_expression function: [
                (identifier) @callee
                (field_expression field: (field_identifier) @callee)
                (scoped_identifier name: (identifier) @callee)
               ])"#,
        ),
        "py" => Some(
            r#"(call function: [
                (identifier) @callee
                (attribute attribute: (identifier) @callee)
               ])"#,
        ),
        "js" | "jsx" | "mjs" | "cjs" | "ts" | "tsx" => Some(
            r#"(call_expression function: [
                (identifier) @callee
                (member_expression property: (property_identifier) @callee)
               ])"#,
        ),
        "go" => Some(
            r#"(call_expression function: [
                (identifier) @callee
                (selector_expression field: (field_identifier) @callee)
               ])"#,
        ),
        "c" | "h" | "cpp" | "cc" | "cxx" | "hpp" | "hh" => {
            Some(r#"(call_expression function: (identifier) @callee)"#)
        }
        _ => None,
    }
}
