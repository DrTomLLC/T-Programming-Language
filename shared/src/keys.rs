/// A simple bit‐mask for keyboard modifiers.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct KeyModifiers(u8);

impl KeyModifiers {
    pub const SHIFT:   Self = Self(0b0001);
    pub const CONTROL: Self = Self(0b0010);
    pub const ALT:     Self = Self(0b0100);
    pub const SUPER:   Self = Self(0b1000);

    /// Combine two sets of modifiers
    #[inline] pub fn union(self, other: Self) -> Self {
        KeyModifiers(self.0 | other.0)
    }

    /// Test whether `flag` is present
    #[inline] pub fn contains(self, flag: Self) -> bool {
        (self.0 & flag.0) != 0
    }
}
