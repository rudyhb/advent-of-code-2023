use std::ops::{Index, IndexMut};
use bitflags::bitflags;
use crate::common::models::Point;

pub struct Grid<T> {
    grid: Box<[Box<[T]>]>,
    len_y: usize,
    len_x: usize,
}

impl<T> Grid<T> {
    #[allow(dead_code)]
    pub fn new(len_x: usize, len_y: usize) -> Self
    where
        T: Default,
    {
        Self {
            grid: (0..len_y)
                .map(|_| (0..len_x).map(|_| T::default()).collect())
                .collect(),
            len_y,
            len_x,
        }
    }
    fn is_in_bounds(&self, x: isize, y: isize) -> bool {
        x >= 0 && x < self.len_x as isize && y >= 0 && y < self.len_y as isize
    }
    pub fn neighbors(&self, point: &Point<usize>, directions: Neighbors) -> Vec<Point<usize>> {
        let mut neighbors = Vec::new();
        const NEIGHBOR_DELTAS: &[(Neighbors, isize, isize)] = &[
            (Neighbors::LEFT, -1, 0),
            (Neighbors::UP, 0, -1),
            (Neighbors::RIGHT, 1, 0),
            (Neighbors::DOWN, 0, 1),
            (Neighbors::UP_LEFT, -1, -1),
            (Neighbors::UP_RIGHT, 1, -1),
            (Neighbors::DOWN_LEFT, -1, 1),
            (Neighbors::DOWN_RIGHT, 1, 1),
        ];
        for &(direction, dx, dy) in NEIGHBOR_DELTAS {
            if directions.contains(direction) {
                let new_x = point.x as isize + dx;
                let new_y = point.y as isize + dy;

                if self.is_in_bounds(new_x, new_y) {
                    neighbors.push(Point {
                        x: new_x as usize,
                        y: new_y as usize,
                    });
                }
            }
        }

        neighbors
    }
    pub fn from_iter<I>(iter: I) -> Self
    where
        I: Iterator,
        I::Item: IntoIterator<Item = T>,
    {
        let grid: Box<[Box<[T]>]> = iter
            .into_iter()
            .map(|row| row.into_iter().collect())
            .collect();
        let len_y = grid.len();
        let len_x = grid.first().map_or(0, |row| row.len());
        assert!(
            grid.iter().all(|row| row.len() == len_x),
            "All rows must be the same length"
        );
        Self { grid, len_x, len_y }
    }
    #[allow(dead_code)]
    pub fn try_from_iter<I, E>(iter: I) -> Result<Self, E>
    where
        I: Iterator,
        I::Item: IntoIterator<Item = Result<T, E>>,
    {
        let grid: Box<[Box<[T]>]> = iter
            .into_iter()
            .map(|row| row.into_iter().collect::<Result<_, E>>())
            .collect::<Result<_, E>>()?;
        let len_y = grid.len();
        let len_x = grid.first().map_or(0, |row| row.len());
        assert!(
            grid.iter().all(|row| row.len() == len_x),
            "All rows must be the same length"
        );
        Ok(Self { grid, len_x, len_y })
    }
    #[allow(dead_code)]
    pub fn len_x(&self) -> usize {
        self.len_x
    }
    #[allow(dead_code)]
    pub fn len_y(&self) -> usize {
        self.len_y
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Neighbors: u8 {
        const UP_LEFT = 0b00000001;
        const UP = 0b00000010;
        const UP_RIGHT = 0b00000100;
        const RIGHT = 0b00001000;
        const DOWN_RIGHT = 0b00010000;
        const DOWN = 0b00100000;
        const DOWN_LEFT = 0b01000000;
        const LEFT = 0b10000000;

        const FOUR_DIRECTIONS = Self::UP.bits() | Self::DOWN.bits() | Self::LEFT.bits() | Self::RIGHT.bits();
        const ALL_DIRECTIONS = 0b11111111;
    }
}

impl<T> Index<usize> for Grid<T> {
    type Output = [T];

    fn index(&self, index: usize) -> &Self::Output {
        &self.grid[index]
    }
}

impl<T> IndexMut<usize> for Grid<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.grid[index]
    }
}

impl<T> Index<&Point<usize>> for Grid<T> {
    type Output = T;

    fn index(&self, index: &Point<usize>) -> &Self::Output {
        &self.grid[index.y][index.x]
    }
}

impl<T> IndexMut<&Point<usize>> for Grid<T> {
    fn index_mut(&mut self, index: &Point<usize>) -> &mut Self::Output {
        &mut self.grid[index.y][index.x]
    }
}
