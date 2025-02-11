use crate::common::models::{Direction, DirectionFlag, Point};
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

pub trait GridLike: Sized {
    type CellType;
    fn len_x(&self) -> usize;
    fn len_y(&self) -> usize;
    fn index_point(&self, i: &Point<usize>) -> &Self::CellType;
    fn is_in_bounds(&self, x: isize, y: isize) -> bool {
        x >= 0 && x < self.len_x() as isize && y >= 0 && y < self.len_y() as isize
    }
    fn neighbors(&self, point: &Point<usize>, directions: DirectionFlag) -> Vec<Point<usize>> {
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
    fn move_in_direction_if(
        &self,
        point: &Point<usize>,
        direction: Direction,
        destination_condition: impl FnOnce((&Point<usize>, &Self::CellType)) -> bool,
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
    fn get(&self, point: &Point<usize>) -> Option<&Self::CellType> {
        if point.x < self.len_x() && point.y < self.len_y() {
            Some(self.index_point(point))
        } else {
            None
        }
    }
    fn iter(&self) -> GridIterator<Self> {
        GridIterator::new(self)
    }
    fn iter_rows(&self) -> GridRowIterator<Self> {
        GridRowIterator::new(self)
    }
    fn display_with_rule<V: Display, F>(&self, rule: F) -> GridDisplayWithRule<Self, V, F>
    where
        F: for<'p> Fn((&'p Point<usize>, &'p Self::CellType)) -> V,
    {
        GridDisplayWithRule {
            grid: self,
            rule,
            _phantom_data: Default::default(),
        }
    }
}

pub struct Grid<T> {
    grid: Box<[Box<[T]>]>,
    len_y: usize,
    len_x: usize,
}

impl<T> Grid<T> {
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
    pub fn swap(&mut self, left: &Point<usize>, right: &Point<usize>) {
        let left: *mut T = &mut self[left];
        let right: *mut T = &mut self[right];
        unsafe {
            std::ptr::swap(left, right);
        }
    }
}

impl<T> GridLike for Grid<T> {
    type CellType = T;

    #[inline]
    fn len_x(&self) -> usize {
        self.len_x
    }

    #[inline]
    fn len_y(&self) -> usize {
        self.len_y
    }

    #[inline]
    fn index_point(&self, i: &Point<usize>) -> &Self::CellType {
        &self[i]
    }
}

impl<T> Index<usize> for Grid<T> {
    type Output = [T];

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.grid[index]
    }
}

impl<T> IndexMut<usize> for Grid<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.grid[index]
    }
}

impl<T> Index<&Point<usize>> for Grid<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: &Point<usize>) -> &Self::Output {
        &self.grid[index.y][index.x]
    }
}

impl<T> IndexMut<&Point<usize>> for Grid<T> {
    #[inline]
    fn index_mut(&mut self, index: &Point<usize>) -> &mut Self::Output {
        &mut self.grid[index.y][index.x]
    }
}

pub struct GridB<'a> {
    data: &'a [u8],
    len_y: usize,
    len_x: usize,
}

impl<'a> GridB<'a> {
    pub fn new(s: &'a str) -> Self {
        const NEWLINE_LEN: usize = '\n'.len_utf8();
        let data = s.trim().as_bytes();
        let len_x = data.iter().position(|&c| c == b'\n').unwrap_or(s.len());
        let len_y = (s.len() + NEWLINE_LEN) / (len_x + NEWLINE_LEN); // s.len() + 1 because of non-existing last \n
        assert_eq!(
            s.len() + NEWLINE_LEN,
            (len_x + NEWLINE_LEN) * len_y,
            "all rows should be same size"
        );

        Self { data, len_y, len_x }
    }
    pub fn display_overriding<'b, V: Display + 'b, F>(
        &'b self,
        overrides: F,
    ) -> GridBDisplay<'b, V, F>
    where
        F: for<'p> Fn(&'p Point<usize>) -> Option<V>,
    {
        GridBDisplay {
            grid: self,
            overrides,
        }
    }
}

impl GridLike for GridB<'_> {
    type CellType = u8;

    #[inline]
    fn len_x(&self) -> usize {
        self.len_x
    }

    #[inline]
    fn len_y(&self) -> usize {
        self.len_y
    }

    #[inline]
    fn index_point(&self, i: &Point<usize>) -> &Self::CellType {
        &self[i]
    }
}

impl Index<usize> for GridB<'_> {
    type Output = [u8];

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index * (self.len_x + 1)..index * (self.len_x + 1) + self.len_x]
    }
}

impl Index<&Point<usize>> for GridB<'_> {
    type Output = u8;

    #[inline]
    fn index(&self, index: &Point<usize>) -> &Self::Output {
        &self[index.y][index.x]
    }
}

pub struct GridIterator<'a, T: GridLike> {
    iterator: GridRowIterator<'a, T>,
    row: Option<RowIterator<'a, T>>,
}

impl<'a, T: GridLike> GridIterator<'a, T> {
    fn new(grid: &'a T) -> Self {
        Self {
            iterator: GridRowIterator::new(grid),
            row: None,
        }
    }
}

impl<'a, T: GridLike> Iterator for GridIterator<'a, T> {
    type Item = (Point<usize>, &'a T::CellType);

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

pub struct GridRowIterator<'a, T: GridLike> {
    grid: &'a T,
    y: usize,
}

impl<'a, T: GridLike> GridRowIterator<'a, T> {
    fn new(grid: &'a T) -> Self {
        Self { grid, y: 0 }
    }
}

impl<'a, T: GridLike> Iterator for GridRowIterator<'a, T> {
    type Item = (usize, RowIterator<'a, T>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.y < self.grid.len_y() {
            let y = self.y;
            self.y += 1;
            Some((y, RowIterator::new(self.grid, y)))
        } else {
            None
        }
    }
    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        if self.grid.len_y() > 0 {
            let y = self.grid.len_y() - 1;
            Some((y, RowIterator::new(self.grid, y)))
        } else {
            None
        }
    }
}

pub struct RowIterator<'a, T: GridLike> {
    grid: &'a T,
    y: usize,
    x: usize,
}

impl<'a, T: GridLike> RowIterator<'a, T> {
    fn new(grid: &'a T, y: usize) -> Self {
        Self { grid, y, x: 0 }
    }
}

impl<'a, T: GridLike> Iterator for RowIterator<'a, T> {
    type Item = (Point<usize>, &'a T::CellType);

    fn next(&mut self) -> Option<Self::Item> {
        if self.x < self.grid.len_x() {
            let point = Point {
                x: self.x,
                y: self.y,
            };
            let value = &self.grid.index_point(&point);
            self.x += 1;
            Some((point, value))
        } else {
            None
        }
    }
}

impl<T: PartialEq> PartialEq for Grid<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len_x == other.len_x
            && self.len_y == other.len_y
            && self
                .iter()
                .zip(other.iter())
                .all(|((_, left), (_, right))| left == right)
    }
}

impl<T: PartialEq + Eq> Eq for Grid<T> {}

impl<T: Eq + PartialEq + Hash> Hash for Grid<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for (_, value) in self.iter() {
            value.hash(state);
        }
    }
}

impl<T: Clone> Clone for Grid<T> {
    fn clone(&self) -> Self {
        Self {
            grid: self.grid.clone(),
            len_y: self.len_y,
            len_x: self.len_x,
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

pub struct GridBDisplay<'a, V: 'a, F>
where
    F: for<'p> Fn(&'p Point<usize>) -> Option<V>,
{
    grid: &'a GridB<'a>,
    overrides: F,
}

impl<'a, V: Display + 'a, F> Display for GridBDisplay<'a, V, F>
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
                    let &value = self.grid.get(&point).unwrap();
                    write!(f, "{}", value as char)?;
                }
            }
        }
        Ok(())
    }
}

pub struct GridDisplayWithRule<'a, T, V: 'a, F>
where
    T: GridLike,
    F: for<'p> Fn((&'p Point<usize>, &'p T::CellType)) -> V,
{
    grid: &'a T,
    rule: F,
    _phantom_data: PhantomData<V>,
}

impl<'a, T, V: Display + 'a, F> Display for GridDisplayWithRule<'a, T, V, F>
where
    T: GridLike,
    F: for<'p> Fn((&'p Point<usize>, &'p T::CellType)) -> V,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.grid.len_y() {
            writeln!(f)?;
            for x in 0..self.grid.len_x() {
                let point = Point { x, y };
                let value = &self.grid.index_point(&point);
                let o = (self.rule)((&point, value));
                write!(f, "{}", o)?;
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

    #[test]
    fn test_grid_iter_bytes() {
        let input = ".....
.012.
.1.3.
.234.
.....";
        let grid = GridB::new(input);
        assert_eq!(25, grid.iter().count());

        let values: Vec<_> = input.chars().filter(|c| !c.is_whitespace()).collect();

        for ((_, &left), &right) in grid.iter().zip(values.iter()) {
            assert_eq!(left as char, right);
        }
    }

    #[test]
    fn test_swap() {
        let mut grid = Grid::from_iter(
            [
                [1, 2, 3].into_iter(),
                [4, 5, 6].into_iter(),
                [7, 8, 9].into_iter(),
            ]
            .into_iter(),
        );
        let left = Point { x: 1, y: 0 };
        let right = Point { x: 0, y: 2 };
        let left_val = grid[&left];
        let right_val = grid[&right];
        println!("[0][1] = {}", grid[&left]);
        println!("[2][0] = {}", grid[&right]);
        grid.swap(&left, &right);
        println!("[0][1] = {}", grid[&left]);
        println!("[2][0] = {}", grid[&right]);
        assert_eq!(left_val, grid[&right]);
        assert_eq!(right_val, grid[&left]);
    }
}
