//! Test categories for targeted test runs.
//!
//! Categories allow filtering tests by feature area, making it easier to
//! focus on specific mechanics during development.
//!
//! Usage with libtest-mimic:
//!   cargo test --test damage_fixtures -- abilities
//!   cargo test --test damage_fixtures -- terrain
//!   cargo test --test damage_fixtures -- gen1

use super::fixtures::DamageTestCase;

/// Test category for grouping related tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    /// Generation-specific tests
    Gen1,
    Gen2,
    Gen3,
    Gen4,
    Gen5,
    Gen6,
    Gen7,
    Gen8,
    Gen9,
    
    /// Feature categories
    Abilities,
    Items,
    Moves,
    Weather,
    Terrain,
    Screens,
    MultiHit,
    Critical,
    Status,
    TypeEffectiveness,
    Tera,
    ZMoves,
    Dynamax,
}

impl Category {
    /// Get all categories that apply to a test case.
    pub fn categorize(case: &DamageTestCase) -> Vec<Category> {
        let mut cats = vec![];
        
        // Generation
        match case.gen {
            1 => cats.push(Category::Gen1),
            2 => cats.push(Category::Gen2),
            3 => cats.push(Category::Gen3),
            4 => cats.push(Category::Gen4),
            5 => cats.push(Category::Gen5),
            6 => cats.push(Category::Gen6),
            7 => cats.push(Category::Gen7),
            8 => cats.push(Category::Gen8),
            9 => cats.push(Category::Gen9),
            _ => {}
        }
        
        let test_name_lower = case.test_name.to_lowercase();
        let id_lower = case.id.to_lowercase();
        
        // Abilities
        if has_ability_keywords(&test_name_lower, &id_lower) {
            cats.push(Category::Abilities);
        }
        
        // Items
        if has_item_keywords(&test_name_lower, &id_lower) {
            cats.push(Category::Items);
        }
        
        // Weather
        if case.field.as_ref().map_or(false, |f| f.weather.is_some()) 
            || test_name_lower.contains("weather")
            || test_name_lower.contains("sun")
            || test_name_lower.contains("rain")
            || test_name_lower.contains("sand")
            || test_name_lower.contains("hail")
            || test_name_lower.contains("snow")
        {
            cats.push(Category::Weather);
        }
        
        // Terrain
        if case.field.as_ref().map_or(false, |f| f.terrain.is_some())
            || test_name_lower.contains("terrain")
        {
            cats.push(Category::Terrain);
        }
        
        // Screens
        if has_screen_keywords(&test_name_lower, case) {
            cats.push(Category::Screens);
        }
        
        // Multi-hit
        if test_name_lower.contains("multi") 
            || test_name_lower.contains("parental bond")
            || case.move_data.hits.is_some()
        {
            cats.push(Category::MultiHit);
        }
        
        // Critical hits
        if case.move_data.is_crit == Some(true) 
            || test_name_lower.contains("crit")
        {
            cats.push(Category::Critical);
        }
        
        // Status
        if case.attacker.status.is_some() 
            || case.defender.status.is_some()
            || test_name_lower.contains("burn")
            || test_name_lower.contains("paralysis")
        {
            cats.push(Category::Status);
        }
        
        // Tera
        if case.attacker.tera_type.is_some() 
            || case.defender.tera_type.is_some()
            || test_name_lower.contains("tera")
            || test_name_lower.contains("stellar")
        {
            cats.push(Category::Tera);
        }
        
        // Z-Moves
        if case.move_data.use_z == Some(true) {
            cats.push(Category::ZMoves);
        }
        
        // Dynamax
        if case.attacker.is_dynamaxed == Some(true) 
            || case.defender.is_dynamaxed == Some(true)
        {
            cats.push(Category::Dynamax);
        }
        
        cats
    }
    
    /// Get the category tag for test naming.
    pub fn tag(&self) -> &'static str {
        match self {
            Category::Gen1 => "gen1",
            Category::Gen2 => "gen2",
            Category::Gen3 => "gen3",
            Category::Gen4 => "gen4",
            Category::Gen5 => "gen5",
            Category::Gen6 => "gen6",
            Category::Gen7 => "gen7",
            Category::Gen8 => "gen8",
            Category::Gen9 => "gen9",
            Category::Abilities => "abilities",
            Category::Items => "items",
            Category::Moves => "moves",
            Category::Weather => "weather",
            Category::Terrain => "terrain",
            Category::Screens => "screens",
            Category::MultiHit => "multihit",
            Category::Critical => "critical",
            Category::Status => "status",
            Category::TypeEffectiveness => "type",
            Category::Tera => "tera",
            Category::ZMoves => "zmove",
            Category::Dynamax => "dynamax",
        }
    }
}

fn has_ability_keywords(test_name: &str, id: &str) -> bool {
    let keywords = [
        "ability", "intimidate", "mold breaker", "levitate", "flash fire",
        "water absorb", "volt absorb", "multitype", "parental bond",
        "weak armor", "mummy", "steely spirit", "quark drive", "protosynthesis",
        "ice scales", "wind rider", "supreme overlord", "gale wings", "triage",
        "power spot", "battery", "flower gift",
    ];
    keywords.iter().any(|k| test_name.contains(k) || id.contains(k))
}

fn has_item_keywords(test_name: &str, id: &str) -> bool {
    let keywords = [
        "item", "plate", "orb", "berry", "choice", "band", "specs", "scarf",
        "life orb", "expert belt", "assault vest",
    ];
    keywords.iter().any(|k| test_name.contains(k) || id.contains(k))
}

fn has_screen_keywords(test_name: &str, case: &DamageTestCase) -> bool {
    let has_screen_in_field = case.field.as_ref().map_or(false, |f| {
        f.defender_side.as_ref().map_or(false, |s| {
            s.is_reflect == Some(true) 
            || s.is_light_screen == Some(true)
            || s.is_aurora_veil == Some(true)
        })
    });
    
    has_screen_in_field
        || test_name.contains("screen")
        || test_name.contains("reflect")
        || test_name.contains("light screen")
        || test_name.contains("aurora veil")
        || test_name.contains("brick break")
        || test_name.contains("psychic fangs")
        || test_name.contains("raging bull")
}

/// Build category tags for a test case (for test naming).
pub fn build_category_tags(case: &DamageTestCase) -> String {
    let cats = Category::categorize(case);
    if cats.is_empty() {
        return String::new();
    }
    
    // Skip generation tag (already in test name), include feature tags
    let feature_tags: Vec<&str> = cats.iter()
        .filter(|c| !matches!(c, Category::Gen1 | Category::Gen2 | Category::Gen3 
            | Category::Gen4 | Category::Gen5 | Category::Gen6 | Category::Gen7 
            | Category::Gen8 | Category::Gen9))
        .map(|c| c.tag())
        .collect();
    
    if feature_tags.is_empty() {
        String::new()
    } else {
        format!("[{}]", feature_tags.join(","))
    }
}
