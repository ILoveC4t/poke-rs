use bitflags::bitflags;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Type {
    Normal = 0,
    Fighting = 1,
    Flying = 2,
    Poison = 3,
    Ground = 4,
    Rock = 5,
    Bug = 6,
    Ghost = 7,
    Steel = 8,
    Fire = 9,
    Water = 10,
    Grass = 11,
    Electric = 12,
    Psychic = 13,
    Ice = 14,
    Dragon = 15,
    Dark = 16,
    Fairy = 17,
    Stellar = 18,
    Unknown = 255,
}

impl Default for Type {
    fn default() -> Self {
        Type::Normal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveCategory {
    Physical,
    Special,
    Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveTarget {
    Normal,
    Self_,
    Any,
    AllAdjacent,
    AllAdjacentFoes,
    AllySide,
    FoeSide,
    All,
    RandomNormal,
    Scripted,
    AllAllies,
    AllyTeam, // Aromatherapy etc
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MoveFlags: u32 {
        const CONTACT = 1 << 0;
        const PROTECT = 1 << 1;
        const MIRROR = 1 << 2;
        const HEAL = 1 << 3;
        const BYPASS_SUB = 1 << 4;
        const BITE = 1 << 5;
        const PUNCH = 1 << 6;
        const SOUND = 1 << 7;
        const POWDER = 1 << 8;
        const BULLET = 1 << 9;
        const PULSE = 1 << 10;
        const WIND = 1 << 11;
        const SLICING = 1 << 12;
        const DANCE = 1 << 13;
        const GRAVITY = 1 << 14;
        const DEFROST = 1 << 15;
        const DISTANCE = 1 << 16;
        const CHARGE = 1 << 17;
        const RECHARGE = 1 << 18;
        const NONSKY = 1 << 19;
        const ALLY_ANIM = 1 << 20;
        const NO_ASSIST = 1 << 21;
        const FAIL_COPYCAT = 1 << 22;
        const FAIL_ENCORE = 1 << 23;
        const FAIL_INSTRUCT = 1 << 24;
        const FAIL_MIMIC = 1 << 25;
        const FAIL_SKETCH = 1 << 26;
        const FUTURE_MOVE = 1 << 27;
        const SNATCH = 1 << 28;
    }
}

#[derive(Debug, Clone)]
pub struct MoveData {
    pub name: &'static str,
    pub type_: Type,
    pub power: u8,
    pub accuracy: Option<u8>,
    pub pp: u8,
    pub priority: i8,
    pub target: MoveTarget,
    pub category: MoveCategory,
    pub flags: MoveFlags,
}

#[derive(Debug, Clone, Copy)]
pub struct Stats {
    pub hp: u16,
    pub atk: u16,
    pub def: u16,
    pub spa: u16,
    pub spd: u16,
    pub spe: u16,
}

#[derive(Debug, Clone)]
pub struct SpeciesData {
    pub name: &'static str,
    pub types: [Type; 2],
    pub base_stats: Stats,
    pub abilities: &'static [&'static str],
    pub weight_kg: f32,
}
