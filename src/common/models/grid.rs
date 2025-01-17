use crate::common::models::{Direction, DirectionFlag, Point};
use std::fmt::{Display, Formatter};
use std::ops::{Index, IndexMut};

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
    pub fn neighbors(&self, point: &Point<usize>, directions: DirectionFlag) -> Vec<Point<usize>> {
        let mut neighbors = Vec::new();
        const NEIGHBOR_DELTAS: &[(DirectionFlag, isize, isize)] = &[
            (DirectionFlag::LEFT, -1, 0),
            (DirectionFlag::UP, 0, -1),
            (DirectionFlag::RIGHT, 1, 0),
            (DirectionFlag::DOWN, 0, 1),
            (DirectionFlag::UP_LEFT, -1, -1),
            (DirectionFlag::UP_RIGHT, 1, -1),
            (DirectionFlag::DOWN_LEFT, -1, 1),
            (DirectionFlag::DOWN_RIGHT, 1, 1),
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
    pub fn move_in_direction_if(
        &self,
        point: &Point<usize>,
        direction: Direction,
        destination_condition: impl FnOnce((&Point<usize>, &T)) -> bool,
    ) -> Option<Point<usize>> {
        point.move_in(direction).and_then(|point| {
            self.get(&point).and_then(|value| {
                if destination_condition((&point, value)) {
                    Some(point)
                } else {
                    None
                }
            })
        })
    }
    pub fn get(&self, point: &Point<usize>) -> Option<&T> {
        if point.x < self.len_x && point.y < self.len_y {
            Some(&self[point])
        } else {
            None
        }
    }
    #[allow(dead_code)]
    pub fn len_x(&self) -> usize {
        self.len_x
    }
    #[allow(dead_code)]
    pub fn len_y(&self) -> usize {
        self.len_y
    }
    pub fn iter(&self) -> GridIterator<T> {
        GridIterator::new(self)
    }
    pub fn iter_rows(&self) -> GridRowIterator<T> {
        GridRowIterator::new(self)
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

impl<T: Default> Default for Point<T> {
    fn default() -> Self {
        Self {
            x: Default::default(),
            y: Default::default(),
        }
    }
}

pub struct GridIterator<'a, T> {
    iterator: GridRowIterator<'a, T>,
    row: Option<RowIterator<'a, T>>,
}

impl<'a, T> GridIterator<'a, T> {
    fn new(grid: &'a Grid<T>) -> Self {
        Self {
            iterator: GridRowIterator::new(grid),
            row: None,
        }
    }
}

impl<'a, T> Iterator for GridIterator<'a, T> {
    type Item = (Point<usize>, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let row = if let Some(row) = self.row.as_mut() {
                row
            } else {
                let (_, next_row) = self.iterator.next()?;
                self.row = Some(next_row);
                self.row.as_mut().unwrap()
            };
            let next = row.next();
            if next.is_some() {
                return next;
            }
            self.row = None;
        }
    }
}

pub struct GridRowIterator<'a, T> {
    grid: &'a Grid<T>,
    y: usize,
}

impl<'a, T> GridRowIterator<'a, T> {
    fn new(grid: &'a Grid<T>) -> Self {
        Self { grid, y: 0 }
    }
}

impl<'a, T> Iterator for GridRowIterator<'a, T> {
    type Item = (usize, RowIterator<'a, T>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.y < self.grid.len_y {
            let y = self.y;
            self.y += 1;
            Some((y, RowIterator::new(self.grid, y)))
        } else {
            None
        }
    }
}

pub struct RowIterator<'a, T> {
    grid: &'a Grid<T>,
    y: usize,
    x: usize,
}

impl<'a, T> RowIterator<'a, T> {
    fn new(grid: &'a Grid<T>, y: usize) -> Self {
        Self { grid, y, x: 0 }
    }
}

impl<'a, T> Iterator for RowIterator<'a, T> {
    type Item = (Point<usize>, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.x < self.grid.len_x {
            let point = Point {
                x: self.x,
                y: self.y,
            };
            let value = &self.grid[&point];
            self.x += 1;
            Some((point, value))
        } else {
            None
        }
    }
}

impl<T: Display> Display for Grid<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            GridDisplay {
                grid: self,
                overrides: |_| None::<&char>,
            }
        )
    }
}

impl<T: Display> Grid<T> {
    pub fn display_overriding<'a, V: Display + 'a, F>(
        &'a self,
        overrides: F,
    ) -> GridDisplay<'a, T, V, F>
    where
        F: for<'p> Fn(&'p Point<usize>) -> Option<V>,
    {
        GridDisplay {
            grid: self,
            overrides,
        }
    }
}

pub struct GridDisplay<'a, T: Display, V: 'a, F>
where
    F: for<'p> Fn(&'p Point<usize>) -> Option<V>,
{
    grid: &'a Grid<T>,
    overrides: F,
}

impl<'a, T: Display, V: Display + 'a, F> Display for GridDisplay<'a, T, V, F>
where
    F: for<'p> Fn(&'p Point<usize>) -> Option<V>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.grid.len_y() {
            writeln!(f)?;
            for x in 0..self.grid.len_x() {
                let point = Point { x, y };
                if let Some(o) = (self.overrides)(&point) {
                    write!(f, "{}", o)?;
                } else {
                    let value = self.grid.get(&point).unwrap();
                    write!(f, "{}", value)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_iter() {
        let input = ".....
.012.
.1.3.
.234.
.....";
        let grid = Grid::from_iter(input.lines().map(|line| line.chars()));
        assert_eq!(25, grid.iter().count());
        for (i, item) in grid.iter() {
            println!("{}, {}", i, item);
        }
    }
}
