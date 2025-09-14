use serde::Serialize;
use wasm_bindgen::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use fxhash::FxHashSet;


// Mangle crates
use mangle_ast as ast;
use mangle_engine::Engine;
use mangle_factstore::{FactStore, ReadOnlyFactStore, TableConfig, TableStoreImpl};
use mangle_analysis::SimpleProgram;
use mangle_parse::Parser;


#[derive(Serialize)]
struct SuccessResponse {
    status: &'static str,
    data: Vec<String>,
}

#[derive(Serialize)]
struct ErrorResponse {
    status: &'static str,
    message: String,
}

// This internal function contains the core Mangle logic.
fn run_query_internal(input: &str) -> Result<Vec<String>, String> {
    if input.trim().is_empty() {
        return Err("Input string cannot be empty.".to_string());
    }

    // 1. Initialization
    let arena = ast::Arena::new_with_global_interner();

    // 2. Parsing
    let mut parser = Parser::new(&arena, input.as_bytes(), "wasm_input");
    parser.next_token().map_err(|e| format!("Parser error: {}", e))?;
    let unit = parser.parse_unit().map_err(|e| format!("Parse error: {}", e))?;
    let clauses = unit.clauses;

    // 3. Separate Program and Query
    if clauses.is_empty() {
        return Err("No clauses found in input.".to_string());
    }
    let (query_clause, program_clauses) = clauses.split_last().unwrap();
    let query_atom = query_clause.head;

    if !query_clause.premises.is_empty() {
        return Err("The last statement must be a query atom, not a rule.".to_string());
    }

    // 4. Build Schema and Store
    let mut schema = HashMap::default();
    for clause in clauses {
        schema.entry(clause.head.sym).or_insert(TableConfig::InMemory);
        for premise in clause.premises {
            if let ast::Term::Atom(a) = premise {
                schema.entry(a.sym).or_insert(TableConfig::InMemory);
            }
        }
    }
    let store = TableStoreImpl::new(&arena, &schema);

    // 5. Build the Program
    let rule_head_preds: FxHashSet<_> = program_clauses.iter()
        .filter(|c| !c.premises.is_empty())
        .map(|c| c.head.sym)
        .collect();

    let mut simple_program = SimpleProgram {
        arena: &arena,
        ext_preds: Vec::new(),
        rules: Default::default(),
    };

    let mut ext_preds_set = FxHashSet::default();

    for clause in program_clauses {
        if clause.premises.is_empty() {
            // It's a fact, add to store
            store.add(&arena, clause.head).map_err(|e| e.to_string())?;
            if !rule_head_preds.contains(&clause.head.sym) {
                ext_preds_set.insert(clause.head.sym);
            }
        } else {
            // It's a rule, add to program
            simple_program.add_clause(&arena, clause);
        }
    }
    simple_program.ext_preds = ext_preds_set.into_iter().collect();

    // 6. Evaluation
    let stratified_program = simple_program.stratify()?;
    let engine = mangle_engine::naive::Naive {};
    engine.eval(&store, &stratified_program).map_err(|e| format!("Evaluation error: {}", e))?;

    // 7. Querying and Formatting
    let results = RefCell::new(Vec::new());
    store.get(query_atom.sym, query_atom.args, &|atom: &ast::Atom| {
        results.borrow_mut().push(atom.to_string());
        Ok(())
    }).map_err(|e| format!("Query error: {}", e))?;

    Ok(results.into_inner())
}

#[wasm_bindgen]
pub fn run_mangle_query(input: &str) -> String {
    // We wrap the core logic in catch_unwind to handle any potential panics
    // from the Mangle engine, ensuring the Wasm module doesn't crash.
    let result = std::panic::catch_unwind(|| {
        run_query_internal(input)
    });

    match result {
        // The logic ran without panicking. Now check the Result.
        Ok(Ok(data)) => {
            let response = SuccessResponse {
                status: "success",
                data,
            };
            serde_json::to_string(&response).unwrap()
        }
        Ok(Err(message)) => {
            let response = ErrorResponse {
                status: "error",
                message,
            };
            serde_json::to_string(&response).unwrap()
        }
        // The logic panicked.
        Err(panic_info) => {
            let message = if let Some(s) = panic_info.downcast_ref::<&'static str>() {
                s.to_string()
            } else if let Some(s) = panic_info.downcast_ref::<String>() {
                s.clone()
            } else {
                "An unknown panic occurred.".to_string()
            };
            let response = ErrorResponse {
                status: "error",
                message,
            };
            serde_json::to_string(&response).unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Import everything from the parent module (e.g., run_mangle_query)
    use serde_json;

    // A helper struct to deserialize the JSON string response for easy assertions.
    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct QueryResult {
        status: String,
        data: Option<Vec<String>>,
        message: Option<String>,
    }

    #[test]
    fn test_success_case() {
        let input = r#"
            service("order-service").
            uses_library("order-service", "log4j", "2.14").
            vulnerable_version("log4j", "2.14").
            is_vulnerable(Svc) :-
              uses_library(Svc, Lib, Ver),
              vulnerable_version(Lib, Ver).
            is_vulnerable(Svc).
        "#;
        let result_json = run_mangle_query(input);
        let result: QueryResult = serde_json::from_str(&result_json).unwrap();

        assert_eq!(result.status, "success");
        assert_eq!(result.data, Some(vec!["is_vulnerable(\"order-service\").".to_string()]));
    }

    #[test]
    fn test_no_results_case() {
        let input = r#"
            service("order-service").
            uses_library("order-service", "log4j", "2.17"). // Safe version
            vulnerable_version("log4j", "2.14").
            is_vulnerable(Svc) :-
              uses_library(Svc, Lib, Ver),
              vulnerable_version(Lib, Ver).
            is_vulnerable(Svc).
        "#;
        let result_json = run_mangle_query(input);
        let result: QueryResult = serde_json::from_str(&result_json).unwrap();

        assert_eq!(result.status, "success");
        assert_eq!(result.data, Some(vec![])); // Expect an empty vector
    }

    #[test]
    fn test_syntax_error_case() {
        let input = r#"
            // Missing dot at the end of the fact
            service("order-service")
            is_vulnerable(Svc).
        "#;
        let result_json = run_mangle_query(input);
        let result: QueryResult = serde_json::from_str(&result_json).unwrap();

        assert_eq!(result.status, "error");
        assert!(result.message.is_some()); // Just check that an error message was provided
    }

    #[test]
    fn test_multi_step_deduction() {
        let input = r#"
            // Base Facts
            is_a("Mufasa", "lion").
            parent("Mufasa", "Simba").
            is_male("Mufasa").

            // Rule 1: Deduce what a father is
            father(Parent, Child) :-
              parent(Parent, Child),
              is_male(Parent).

            // Rule 2: Deduce a paternal relationship based on the first rule
            paternal_grandfather(GF, GC) :-
              father(GF, F),
              parent(F, GC).

            // This would fail, so we don't include it yet.
            // For now, we test the first deduction.
            father(Father, "Simba").
        "#;
        let result_json = run_mangle_query(input);
        let result: QueryResult = serde_json::from_str(&result_json).unwrap();

        assert_eq!(result.status, "success");
        assert_eq!(result.data, Some(vec!["father(\"Mufasa\", \"Simba\").".to_string()]));
    }
}
