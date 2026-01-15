use crate::behavior::Orientation;

#[derive(Clone, Debug, Default, Copy)]
pub struct CollisionFieldData {
    pub player_free: bool,
    pub ghost_free: bool,
    pub see_through: bool,
    pub wall_orientation: Orientation,
    pub is_dynamic: bool,
    pub stair_offset: i32,
}
