use crate::components::board::boardposition::BoardPosition;

use super::fielddata::LightFieldData;

#[derive(Clone, Debug)]
pub struct LightFieldSector {
    field: Vec<LightFieldData>,
    min_x: i64,
    min_y: i64,
    _min_z: i64,
    sz_x: usize,
    sz_y: usize,
    _sz_z: usize,
}

// FIXME: This has exactly the same computation as HashMap, at least for the part
// that it matters.
impl LightFieldSector {
    pub fn new(min_x: i64, min_y: i64, min_z: i64, max_x: i64, max_y: i64, max_z: i64) -> Self {
        let sz_x = (max_x - min_x + 1).max(0) as usize;
        let sz_y = (max_y - min_y + 1).max(0) as usize;
        let sz_z = (max_z - min_z + 1).max(0) as usize;
        let light_field: Vec<LightFieldData> =
            vec![LightFieldData::default(); sz_x * sz_y * sz_z + 15000];
        Self {
            field: light_field,
            min_x,
            min_y,
            _min_z: min_z,
            sz_x,
            sz_y,
            _sz_z: sz_z,
        }
    }

    #[inline]
    fn vec_coord(&self, x: i64, y: i64, _z: i64) -> usize {
        let x = x - self.min_x;
        let y = y - self.min_y;

        // let z = z - self.min_z; These are purposefully allowing overflow and clamping
        // to an out of bounds value.
        let x = (x as usize).min(self.sz_x);
        let y = (y as usize).min(self.sz_y);

        // let z = (z as usize).min(self.sz_z);
        //
        // * z * self.sz_x * self.sz_y
        x + y * self.sz_x
        // (x & 0xF) | ((y & 0xF) << 4) | ((x & 0xFFFFF0) << 4) | ((y & 0xFFFFF0) << 8)
    }

    pub fn get_mut(&mut self, x: i64, y: i64, z: i64) -> Option<&mut LightFieldData> {
        let xyz = self.vec_coord(x, y, z);
        self.field.get_mut(xyz)
    }

    pub fn get_pos(&self, p: &BoardPosition) -> Option<&LightFieldData> {
        self.get(p.x, p.y, p.z)
    }

    pub fn get_mut_pos(&mut self, p: &BoardPosition) -> Option<&mut LightFieldData> {
        self.get_mut(p.x, p.y, p.z)
    }

    #[inline]
    pub fn get(&self, x: i64, y: i64, z: i64) -> Option<&LightFieldData> {
        let xyz = self.vec_coord(x, y, z);
        self.field.get(xyz)
    }

    /// get_pos_unchecked: Does not seem to be any faster.
    // #[inline] pub unsafe fn get_pos_unchecked(&self, p: &BoardPosition) ->
    // &LightFieldData { // let xyz = self.vec_coord(p.x, p.y, p.z); let xyz = (p.x -
    // self.min_x) as usize + (p.y - self.min_y) as usize * self.sz_x;
    // self.field.get_unchecked(xyz) }
    pub fn insert(&mut self, x: i64, y: i64, z: i64, lfd: LightFieldData) {
        let xyz = self.vec_coord(x, y, z);
        self.field[xyz] = lfd;
    }
}
