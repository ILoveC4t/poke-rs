//! Data-driven damage calculation tests.
//!
//! Uses `libtest-mimic` to generate individual tests from fixtures,
//! allowing filtering with `cargo test Arceus` etc.
//!
//! # Filtering Examples
//!
//! ```sh
//! # Run all tests
//! cargo test --test damage_fixtures
//!
//! # Filter by generation
//! cargo test --test damage_fixtures -- gen9
//!
//! # Filter by test name
//! cargo test --test damage_fixtures -- Arceus
//! cargo test --test damage_fixtures -- "Brick Break"
//!
//! # Filter by category (in test name)
//! cargo test --test damage_fixtures -- abilities
//! cargo test --test damage_fixtures -- terrain
//! cargo test --test damage_fixtures -- screens
//! ```

mod common;

use common::fixtures::DamageFixture;
use common::helpers::{run_damage_test, sanitize_name};
use common::skip_list::should_skip;
use common::categories::build_category_tags;

use libtest_mimic::{Arguments, Trial, Failed};

fn main() {
    let args = Arguments::from_args();
    
    let fixture = match DamageFixture::load() {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to load fixtures: {}", e);
            std::process::exit(1);
        }
    };
    
    println!("Loaded {} damage test cases", fixture.cases.len());
    
    let tests: Vec<Trial> = fixture.cases.into_iter().map(|case| {
        // Build test name with generation and categories
        // Format: gen{N}::{category_tags}::{test_name}::{id}
        let category_tags = build_category_tags(&case);
        let test_name = if category_tags.is_empty() {
            format!(
                "gen{}::{}::{}",
                case.gen,
                sanitize_name(&case.test_name),
                sanitize_name(&case.id)
            )
        } else {
            format!(
                "gen{}::{}::{}::{}",
                case.gen,
                category_tags,
                sanitize_name(&case.test_name),
                sanitize_name(&case.id)
            )
        };
        
        // Check if this fixture should be skipped
        if should_skip(&case.id) {
            Trial::test(test_name, || Ok(())).with_ignored_flag(true)
        } else {
            Trial::test(test_name, move || {
                run_damage_test(&case).map_err(|e| Failed::from(e))
            })
        }
    }).collect();
    
    println!("Running {} tests", tests.len());
    
    libtest_mimic::run(&args, tests).exit();
}
