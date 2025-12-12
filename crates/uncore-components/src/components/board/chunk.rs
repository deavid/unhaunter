use std::ops::Range;

use super::boardposition::BoardPosition;

pub const CHUNK_SIZE_X: usize = 8;
pub const CHUNK_SIZE_Y: usize = 8;
pub const CHUNK_SIZE_Z: usize = 1;

#[derive(Debug, Clone)]
pub struct ChunkIterator {
    map_size: (usize, usize, usize),
    current_chunk_x: usize,
    current_chunk_y: usize,
    current_chunk_z: usize,
    max_chunk_x: usize,
    max_chunk_y: usize,
    max_chunk_z: usize,
}

impl ChunkIterator {
    pub fn new(map_size: (usize, usize, usize)) -> Self {
        ChunkIterator {
            map_size,
            current_chunk_x: 0,
            current_chunk_y: 0,
            current_chunk_z: 0,
            max_chunk_x: map_size.0.div_ceil(CHUNK_SIZE_X),
            max_chunk_y: map_size.1.div_ceil(CHUNK_SIZE_Y),
            max_chunk_z: map_size.2.div_ceil(CHUNK_SIZE_Z),
        }
    }

    pub fn get_chunk_ranges(&self) -> (Range<usize>, Range<usize>, Range<usize>) {
        let start_x = self.current_chunk_x * CHUNK_SIZE_X;
        let end_x = (start_x + CHUNK_SIZE_X).min(self.map_size.0);
        let start_y = self.current_chunk_y * CHUNK_SIZE_Y;
        let end_y = (start_y + CHUNK_SIZE_Y).min(self.map_size.1);
        let start_z = self.current_chunk_z * CHUNK_SIZE_Z;
        let end_z = (start_z + CHUNK_SIZE_Z).min(self.map_size.2);

        (start_x..end_x, start_y..end_y, start_z..end_z)
    }
}

impl Iterator for ChunkIterator {
    type Item = (Range<usize>, Range<usize>, Range<usize>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_chunk_z >= self.max_chunk_z {
            return None;
        }

        let ranges = self.get_chunk_ranges();

        self.current_chunk_x += 1;
        if self.current_chunk_x >= self.max_chunk_x {
            self.current_chunk_x = 0;
            self.current_chunk_y += 1;
            if self.current_chunk_y >= self.max_chunk_y {
                self.current_chunk_y = 0;
                self.current_chunk_z += 1;
            }
        }

        Some(ranges)
    }
}

/// Convenience function to get a BoardPosition from the chunk's *start* corner
pub fn chunk_start_bpos(chunk_x: usize, chunk_y: usize, chunk_z: usize) -> BoardPosition {
    BoardPosition {
        x: (chunk_x * CHUNK_SIZE_X) as i64,
        y: (chunk_y * CHUNK_SIZE_Y) as i64,
        z: (chunk_z * CHUNK_SIZE_Z) as i64,
    }
}

/// Iterator that takes a (Range<usize>, Range<usize>, Range<usize>) from the ChunkIterator
/// and iterates in all 3 dimensions, returning the global position of each cell.
pub struct CellIterator {
    x_range: Range<usize>,
    y_range: Range<usize>,
    z_range: Range<usize>,
    current_x: usize,
    current_y: usize,
    current_z: usize,
}

impl CellIterator {
    pub fn new(ranges: &(Range<usize>, Range<usize>, Range<usize>)) -> Self {
        CellIterator {
            x_range: ranges.0.clone(),
            y_range: ranges.1.clone(),
            z_range: ranges.2.clone(),
            current_x: ranges.0.start,
            current_y: ranges.1.start,
            current_z: ranges.2.start,
        }
    }
}

impl Iterator for CellIterator {
    type Item = (usize, usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_z >= self.z_range.end {
            return None;
        }

        let position = (self.current_x, self.current_y, self.current_z);

        self.current_x += 1;
        if self.current_x >= self.x_range.end {
            self.current_x = self.x_range.start;
            self.current_y += 1;
            if self.current_y >= self.y_range.end {
                self.current_y = self.y_range.start;
                self.current_z += 1;
            }
        }

        Some(position)
    }
}
