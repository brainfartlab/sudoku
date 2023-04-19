use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use bitvec::prelude::*;

const SUDOKU_L: u32 = 9;
const SUDOKU_C: u32 = 3;
const SUDOKU_A: usize = 81;

#[derive(Debug, PartialEq)]
struct Mask {
    bits: BitArr!(for SUDOKU_A, in u32),
}

impl Mask {
    fn new() -> Self {
        Self { bits: bitarr!(u32, Lsb0; 0; 81) }
    }

    fn cell(index: u32) -> Self {
        assert!(index < SUDOKU_A as u32 && index % SUDOKU_L < 7);

        let mut bits = bitarr!(u32, Lsb0;
            1, 1, 1, 0, 0, 0, 0, 0, 0,
            1, 1, 1, 0, 0, 0, 0, 0, 0,
            1, 1, 1, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        );

        bits.shift_right(index as usize);

        Self { bits }
    }

    fn row(index: u32) -> Self {
        assert!(index % SUDOKU_L == 0);

        let mut bits = bitarr!(u32, Lsb0;
            1, 1, 1, 1, 1, 1, 1, 1, 1,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        );

        bits.shift_right(index as usize);

        Self { bits }
    }

    fn column(index: u32) -> Self {
        assert!(index < SUDOKU_L);

        let mut bits = bitarr!(u32, Lsb0;
            1, 0, 0, 0, 0, 0, 0, 0, 0,
            1, 0, 0, 0, 0, 0, 0, 0, 0,
            1, 0, 0, 0, 0, 0, 0, 0, 0,
            1, 0, 0, 0, 0, 0, 0, 0, 0,
            1, 0, 0, 0, 0, 0, 0, 0, 0,
            1, 0, 0, 0, 0, 0, 0, 0, 0,
            1, 0, 0, 0, 0, 0, 0, 0, 0,
            1, 0, 0, 0, 0, 0, 0, 0, 0,
            1, 0, 0, 0, 0, 0, 0, 0, 0,
        );

        bits.shift_right(index as usize);

        Self { bits }
    }
}

impl Deref for Mask {
    type Target = BitSlice<u32, Lsb0>;

    fn deref(&self) -> &Self::Target {
        &self.bits[..SUDOKU_A]
    }
}

impl DerefMut for Mask {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bits[..SUDOKU_A]
    }
}

#[derive(Debug)]
struct CorruptLayerError {
    value: u32,
}

#[derive(Debug)]
struct Layer {
    indices: Vec<u32>,
    mask: Mask,
    value: u32,
}

impl Layer {
    fn new(value: u32) -> Self {
        Self {
            indices: vec![],
            mask: Mask::new(),
            value,
        }
    }

    fn blot(&mut self, index: u32) {
        self.mask.set(index as usize, true);
    }

    fn occupy(&mut self, index: u32) {
        let mask = Mask::cell(SUDOKU_C*SUDOKU_L * (index / (SUDOKU_C*SUDOKU_L)) + SUDOKU_C*((index % SUDOKU_L) / SUDOKU_C));
        *self.mask |= mask.bits;

        let mask = Mask::row(SUDOKU_L*(index / SUDOKU_L));
        *self.mask |= mask.bits;

        let mask = Mask::column(index % SUDOKU_L);
        *self.mask |= mask.bits;

        self.indices.push(index);
    }

    fn is_solved(&self) -> bool {
        self.indices.len() == SUDOKU_L as usize
    }
}

pub struct Puzzle {
    layers: [Layer; SUDOKU_L as usize],
}

impl Puzzle {
    pub fn parse(feed: &str) -> Self {
        let mut layers: Vec<Layer> = (1..=SUDOKU_L)
            .map(|i| Layer::new(i))
            .collect();

        feed
            .chars()
            .map(|c| c.to_digit(10).unwrap())
            .enumerate()
            .filter(|&(_, value)| value != 0)
            .for_each(|(index, value)| {
                layers
                    .iter_mut()
                    .for_each(|layer| {
                        if value == layer.value {
                            layer.occupy(index as u32);
                        } else {
                            layer.blot(index as u32);
                        };
                    });
            });

        Self {
            layers: layers.try_into().unwrap(),
        }
    }

    fn update(&mut self, value: u32, index: u32) {
        self.layers
            .iter_mut()
            .for_each(|layer| {
                if layer.value == value {
                    layer.occupy(index);
                } else {
                    layer.blot(index);
                };
            });
    }

    fn is_solved(&self) -> bool {
        self.layers.iter().all(|layer| layer.is_solved())
    }

    pub fn solve(&mut self) {
        let mut segments: Vec<Box<dyn Segment>> = vec![];

        for index in 1..=9 {
            segments.push(Box::new(Row::new(index)));
            segments.push(Box::new(Column::new(index)));

            let i = (index - 1) / SUDOKU_C + 1;
            let j = (index - 1) % SUDOKU_C + 1;
            segments.push(Box::new(Cell::new(i, j)));
        }

        while !self.is_solved() {
            segments.sort_by_key(|segment| SUDOKU_L as usize - segment.count_open(&self));

            segments
                .iter()
                .for_each(|segment| segment.iterate(self));

            segments.retain(|segment| segment.count_open(&self) > 0);
        }
    }

    pub fn readout(&self) -> String {
        let mut readout = "0".repeat(SUDOKU_A);

        let positions: HashMap<u32, u32> = self.layers
            .iter()
            .flat_map(|layer| {
                layer.indices.iter().map(|&index| (index, layer.value))
            })
            .collect();

        for (index, value) in positions {
            let index = index as usize;
            readout.replace_range(index..index+1, &value.to_string());
        }

        readout
    }
}

trait Segment {
    fn count_layer_positions(&self, layer: &Layer) -> usize;
    fn locate(&self, layer: &Layer) -> Option<u32>;

    fn is_layer_solved(&self, layer: &Layer) -> Result<bool, CorruptLayerError> {
        match self.count_layer_positions(layer) {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(CorruptLayerError {
                value: layer.value,
            }),
        }
    }

    fn count_open(&self, puzzle: &Puzzle) -> usize {
        SUDOKU_L as usize - puzzle.layers
            .iter()
            .filter(|layer| self.is_layer_solved(layer).unwrap())
            .count()
    }

    fn iterate(&self, puzzle: &mut Puzzle) {
        let mut finds = vec![];

        puzzle.layers
            .iter()
            .filter(|layer| !self.is_layer_solved(layer).expect("Found corrupted layer!"))
            .for_each(|layer| {
                match self.locate(layer) {
                    Some(index) => finds.push((layer.value, index)),
                    None => (),
                };
            });

        finds
            .iter()
            .for_each(|&(value, index)| {
                puzzle.update(value, index);
            });
    }
}

#[derive(Debug)]
struct Row {
    index: u32,
}

impl Row {
    fn new(row: u32) -> Self {
        assert!(row >= 1 && row <= SUDOKU_L);
        Self { index: SUDOKU_L * (row - 1) }
    }
}

impl Segment for Row {
    fn locate(&self, layer: &Layer) -> Option<u32> {
        let mut row_mask = Mask::row(self.index);
        *row_mask &= &*layer.mask;

        match row_mask.count_ones() {
            8 => {
                let s = self.index as usize;
                let index = row_mask[s..s + SUDOKU_L as usize].first_zero().unwrap() + s;
                Some(index as u32)
            },
            _ => None,
        }
    }

    fn count_layer_positions(&self, layer: &Layer) -> usize {
        layer.indices
            .iter()
            .filter(|&&i| i >= SUDOKU_L*self.index && i < SUDOKU_L*(self.index + 1))
            .count()
    }
}

#[derive(Debug)]
struct Column {
    index: u32,
}

impl Column {
    fn new(column: u32) -> Self {
        assert!(column >= 1 && column <= SUDOKU_L);
        Self { index: column - 1 }
    }
}

impl Segment for Column {
    fn locate(&self, layer: &Layer) -> Option<u32> {
        let mut column_mask = Mask::column(self.index);
        *column_mask &= &*layer.mask;

        match column_mask.count_ones() {
            8 => {
                let l = column_mask
                    .iter_ones()
                    .enumerate()
                    .take_while(|(i, index)| self.index as usize + (SUDOKU_L as usize)*i == *index)
                    .last();

                let s = match l {
                    Some((_, index)) => index as u32 + SUDOKU_L,
                    None => self.index,
                };

                Some(s)
            },
            _ => None,
        }
    }

    fn count_layer_positions(&self, layer: &Layer) -> usize {
        layer.indices
            .iter()
            .filter(|&&i| i % SUDOKU_L == self.index)
            .count()
    }
}

#[derive(Debug)]
struct Cell {
    index: u32,
}

impl Cell {
    fn new(i: u32, j: u32) -> Self {
        assert!(i >= 1 && i <= SUDOKU_C);
        assert!(j >= 1 && j <= SUDOKU_C);
        Self { index: SUDOKU_C*SUDOKU_L*(i - 1) + SUDOKU_C*(j - 1) }
    }
}

impl Segment for Cell {
    fn locate(&self, layer: &Layer) -> Option<u32> {
        let mut cell_mask = Mask::cell(self.index);
        *cell_mask &= &*layer.mask;

        match cell_mask.count_ones() {
            8 => {
                let l = cell_mask
                    .iter_ones()
                    .enumerate()
                    .take_while(|&(i, index)| self.index + SUDOKU_L*(i as u32 / SUDOKU_C) + i as u32 % SUDOKU_C == index as u32)
                    .last();

                let s = match l {
                    Some((i, _)) => {
                        let i = i as u32 + 1;
                        self.index + SUDOKU_L*(i / SUDOKU_C) + i % SUDOKU_C
                    },
                    None => self.index,
                };

                Some(s)
            },
            _ => None,
        }
    }

    fn count_layer_positions(&self, layer: &Layer) -> usize {
        layer.indices
            .iter()
            .filter(|&&i| {
                if i >= self.index && i < self.index + (SUDOKU_C - 1)*SUDOKU_L + SUDOKU_C {
                    (i - self.index) % SUDOKU_L < SUDOKU_C
                } else {
                    false
                }
            })
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_new() {
        let mask = Mask::new();
        assert_eq!(mask.deref(), bits![
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);
    }

    #[test]
    fn mask_cell() {
        let mask = Mask::cell(3);
        assert_eq!(mask.deref(), bits![
            0, 0, 0, 1, 1, 1, 0, 0, 0,
            0, 0, 0, 1, 1, 1, 0, 0, 0,
            0, 0, 0, 1, 1, 1, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);
    }

    #[test]
    fn mask_row() {
        let mask = Mask::row(18);
        assert_eq!(mask.deref(), bits![
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            1, 1, 1, 1, 1, 1, 1, 1, 1,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);
    }

    #[test]
    fn mask_column() {
        let mask = Mask::column(8);
        assert_eq!(mask.deref(), bits![
            0, 0, 0, 0, 0, 0, 0, 0, 1,
            0, 0, 0, 0, 0, 0, 0, 0, 1,
            0, 0, 0, 0, 0, 0, 0, 0, 1,
            0, 0, 0, 0, 0, 0, 0, 0, 1,
            0, 0, 0, 0, 0, 0, 0, 0, 1,
            0, 0, 0, 0, 0, 0, 0, 0, 1,
            0, 0, 0, 0, 0, 0, 0, 0, 1,
            0, 0, 0, 0, 0, 0, 0, 0, 1,
            0, 0, 0, 0, 0, 0, 0, 0, 1,
        ]);
    }

    #[test]
    fn layer_blot() {
        let mut layer = Layer::new(1);
        layer.blot(0);
        layer.blot(34);
        layer.blot(80);

        assert_eq!(layer.mask.deref(), bits![
            1, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 1, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 1,
        ]);
    }

    #[test]
    fn layer_occupy() {
        let mut layer = Layer::new(1);
        layer.occupy(0);
        layer.occupy(34);
        layer.occupy(80);

        assert_eq!(layer.mask.deref(), bits![
            1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 0, 0, 0, 0, 1, 1,
            1, 1, 1, 0, 0, 0, 0, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 0, 0, 0, 0, 0, 1, 1, 1,
            1, 0, 0, 0, 0, 0, 1, 1, 1,
            1, 0, 0, 0, 0, 0, 1, 1, 1,
            1, 0, 0, 0, 0, 0, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1,
        ]);
    }

    #[test]
    fn puzzle_parse() {
        let puzzle = Puzzle::parse("029306807000702050607100002005009010000080000004610000040060080061874209708031005");

        assert_eq!(puzzle.layers[0].indices.len(), 5);
        assert_eq!(puzzle.layers[1].indices.len(), 4);
        assert_eq!(puzzle.layers[2].indices.len(), 2);
        assert_eq!(puzzle.layers[3].indices.len(), 3);
        assert_eq!(puzzle.layers[4].indices.len(), 3);
        assert_eq!(puzzle.layers[5].indices.len(), 5);
        assert_eq!(puzzle.layers[6].indices.len(), 5);
        assert_eq!(puzzle.layers[7].indices.len(), 5);
        assert_eq!(puzzle.layers[8].indices.len(), 3);
    }

    #[test]
    fn segment_row_count_open() {
        let puzzle = Puzzle::parse("029306807000702050607100002005009010000080000004610000040060080061874209708031005");

        let row = Row::new(1);
        assert_eq!(row.count_open(&puzzle), 3);
    }

    #[test]
    fn segment_column_count_open() {
        let puzzle = Puzzle::parse("029306807000702050607100002005009010000080000004610000040060080061874209708031005");

        let column = Column::new(1);
        assert_eq!(column.count_open(&puzzle), 7);
    }

    #[test]
    fn segment_cell_count_open() {
        let puzzle = Puzzle::parse("029306807000702050607100002005009010000080000004610000040060080061874209708031005");

        let cell = Cell::new(1, 1);
        assert_eq!(cell.count_open(&puzzle), 5);
    }

    #[test]
    fn segment_cell_count_layer_positions() {
        let puzzle = Puzzle::parse("029306807000702056607100002005009610000080000004610000040060080061874209708031005");

        let cell = Cell::new(1, 3);
        assert_eq!(cell.count_layer_positions(&puzzle.layers[5]), 1);
    }

    #[test]
    fn segment_row_locate_1() {
        // In the first row we can place a 1 at the first position
        let puzzle = Puzzle::parse("029306807000702050607100002005009010000080000004610000040060080061874209708031005");

        let row = Row::new(1);
        assert_eq!(row.locate(&puzzle.layers[0]), Some(0));
    }

    #[test]
    fn segment_column_locate_2() {
        // In the fifth column we can place a 2 at the fourth position
        let puzzle = Puzzle::parse("029306807000702050607100002005009010000080000004610000040060080061874209708031005");

        let column = Column::new(5);
        assert_eq!(column.locate(&puzzle.layers[1]), Some(31));
    }

    #[test]
    fn segment_cell_locate_7() {
        // In the bottom right cell we can place a 7 at the first position
        let puzzle = Puzzle::parse("029306807000702050607100002005009010000080000004610000040060080061874209708031005");

        let cell = Cell::new(3, 3);
        assert_eq!(cell.locate(&puzzle.layers[6]), Some(60));
    }

    #[test]
    fn puzzle_readout() {
        // In the bottom right cell we can place a 7 at the first position
        let puzzle = Puzzle::parse("029306807000702050607100002005009010000080000004610000040060080061874209708031005");

        assert_eq!(puzzle.readout(), String::from("029306807000702050607100002005009010000080000004610000040060080061874209708031005"));
    }

    #[test]
    fn puzzle_row_iteration() {
        // In the bottom right cell we can place a 7 at the first position
        let mut puzzle = Puzzle::parse("029306807000702050607100002005009010000080000004610000040060080061874209708031005");

        let row = Row::new(1);
        row.iterate(&mut puzzle);
        assert_eq!(puzzle.readout(), String::from("129306807000702050607100002005009010000080000004610000040060080061874209708031005"));
    }

    #[test]
    fn puzzle_solve() {
        // In the bottom right cell we can place a 7 at the first position
        let mut puzzle = Puzzle::parse("029306807000702050607100002005009010000080000004610000040060080061874209708031005");

        puzzle.solve();
        assert_eq!(puzzle.readout(), String::from("129356847483792156657148392875429613216583974934617528342965781561874239798231465"));
    }
}
