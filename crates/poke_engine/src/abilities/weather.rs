//! Weather and Terrain definitions.

/// Weather types for ability hooks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Weather {
    None = 0,
    Sun = 1,
    Rain = 2,
    Sand = 3,
    Hail = 4,
    Snow = 5,        // Gen 9
    HarshSun = 6,    // Primal
    HeavyRain = 7,   // Primal
    StrongWinds = 8, // Delta Stream
}

/// Terrain types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Terrain {
    None = 0,
    Electric = 1,
    Grassy = 2,
    Misty = 3,
    Psychic = 4,
}
