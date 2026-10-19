use crate::config::Modifier;

/// A set of modifier keys packed into one byte, so the hook callback can compare it without
/// allocating or locking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct ModifierSet(u8);

impl ModifierSet {
    pub const EMPTY: Self = Self(0);

    pub const fn bits(self) -> u8 {
        self.0
    }

    pub const fn from_bits(bits: u8) -> Self {
        Self(bits & 0b1111)
    }

    #[must_use]
    pub const fn with(self, modifier: Modifier) -> Self {
        Self(self.0 | Self::bit(modifier))
    }

    pub const fn contains(self, modifier: Modifier) -> bool {
        self.0 & Self::bit(modifier) != 0
    }

    const fn bit(modifier: Modifier) -> u8 {
        match modifier {
            Modifier::Ctrl => 0b0001,
            Modifier::Shift => 0b0010,
            Modifier::Alt => 0b0100,
            Modifier::Win => 0b1000,
        }
    }
}

impl FromIterator<Modifier> for ModifierSet {
    fn from_iter<I: IntoIterator<Item = Modifier>>(modifiers: I) -> Self {
        modifiers.into_iter().fold(Self::EMPTY, Self::with)
    }
}
