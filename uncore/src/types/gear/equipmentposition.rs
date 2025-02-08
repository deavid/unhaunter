use enum_iterator::Sequence;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EquipmentPosition {
    Hand(Hand),
    Stowed,
    // Van,
    Deployed,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Sequence)]
pub enum Hand {
    Left,
    Right,
}
