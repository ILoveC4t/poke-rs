//! Type-safe damage modifier.

/// A fixed-point damage modifier (4096 scale).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Modifier(pub u16);

impl Modifier {
    /// 1.0x modifier (4096).
    pub const ONE: Self = Self(4096);

    /// 0.5x modifier (2048).
    pub const HALF: Self = Self(2048);

    /// 2.0x modifier (8192).
    pub const DOUBLE: Self = Self(8192);

    /// 1.5x modifier (6144).
    pub const ONE_POINT_FIVE: Self = Self(6144);

    /// 1.2x modifier (4915).
    pub const ONE_POINT_TWO: Self = Self(4915);

    /// 1.3x modifier (5325).
    /// Note: This is the standard 1.3x used by Sheer Force, Tough Claws, etc.
    /// Life Orb uses a slightly different value (5324).
    pub const ONE_POINT_THREE: Self = Self(5325);

    /// Life Orb modifier (5324, approx 1.3x).
    /// Note: 1.3 * 4096 = 5324.8, but Life Orb uses 5324 in Gen 5+.
    pub const LIFE_ORB: Self = Self(5324);

    /// Screens in Doubles (Reflect/Light Screen/Aurora Veil).
    /// Value is 2732 (approx 2/3).
    pub const SCREENS_DOUBLES: Self = Self(2732);

    /// Filter/Solid Rock/Prism Armor (0.75x).
    pub const FILTER: Self = Self(3072);

    /// Create a new modifier from a raw u16 value.
    pub const fn new(val: u16) -> Self {
        Self(val)
    }

    /// Get the raw u16 value.
    pub const fn val(self) -> u16 {
        self.0
    }
}

/// Macro to create a Modifier from a float literal at compile time.
///
/// Rounds to the nearest integer: `round(val * 4096)`.
///
/// # Example
/// ```rust
/// use poke_engine::modifier;
/// const MOD: poke_engine::damage::Modifier = modifier!(1.5); // Modifier(6144)
/// ```
#[macro_export]
macro_rules! modifier {
    ($val:expr) => {
        $crate::damage::Modifier::new(($val * 4096.0 + 0.5) as u16)
    };
}
