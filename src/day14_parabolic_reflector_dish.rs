use crate::common::day_setup::Day;
use crate::common::models::grid::GridLike;
use crate::common::models::{Direction, Grid, Point};
use std::collections::{HashMap, VecDeque};
use std::str::FromStr;

pub fn day() -> Day {
    Day::new(run).with_test_inputs(&["O....#....
O.OO#....#
.....##...
OO.#O....O
.O.....O#.
O.#..O.#.#
..O..#O..O
.......O..
#....###..
#OO..#...."])
}
pub fn run(input: &str) {
    let mut platform: Platform = input.parse().unwrap();
    log::debug!(
        "pre-tilt:\n{}",
        platform.grid.display_with_rule(display_grid)
    );
    platform.tilt(Direction::Up);
    log::debug!(
        "post-tilt:\n{}",
        platform.grid.display_with_rule(display_grid)
    );
    println!("part 1 total load: {}", platform.total_load());

    let target = 1000000000;

    let platform: Platform = input.parse().unwrap();
    print_load_after(platform, target);
}

fn print_load_after(mut platform: Platform, target: usize) {
    let mut cache = HashMap::new();
    let mut indexed = Vec::new();
    for i in 0..10000usize {
        if let Some(existing) = cache.get(&platform) {
            log::debug!("looped at i={} and {}", existing, i);
            let j = (target - existing) % (i - existing);
            let platform: &Platform = &indexed[j + existing];
            log::debug!(
                "after {} cycles:\n{}",
                target,
                platform.grid.display_with_rule(display_grid)
            );
            println!(
                "part 2 total load after {} cycles: {}",
                target,
                platform.total_load()
            );
            return;
        }
        cache.insert(platform.clone(), i);
        indexed.push(platform.clone());

        for direction in [
            Direction::Up,
            Direction::Left,
            Direction::Down,
            Direction::Right,
        ] {
            platform.tilt(direction);
        }

        if i < 3 {
            log::debug!(
                "after {} cycles:\n{}",
                i + 1,
                platform.grid.display_with_rule(display_grid)
            );
        }
    }
    panic!("max iterations exceeded");
}

fn display_grid(space: (&Point<usize>, &Option<Rock>)) -> char {
    match space.1 {
        None => '.',
        Some(Rock::Round) => 'O',
        Some(Rock::Square) => '#',
    }
}

#[derive(Clone, Hash, Eq, PartialEq)]
struct Platform {
    grid: Grid<Option<Rock>>,
}

impl Platform {
    pub fn tilt(&mut self, direction: Direction) {
        let ([outer, inner], to_point) = self.get_ranges(direction);

        for outer in outer {
            let mut empty = VecDeque::new();
            for inner in inner.box_clone() {
                let point = to_point(outer, inner);
                match self.grid[&point] {
                    None => {
                        empty.push_back(point);
                    }
                    Some(Rock::Square) => {
                        empty.clear();
                    }
                    Some(Rock::Round) => {
                        if let Some(available) = empty.pop_front() {
                            self.grid.swap(&point, &available);
                            empty.push_back(point);
                        }
                    }
                }
            }
        }
    }
    pub fn total_load(&self) -> u64 {
        self.grid
            .iter()
            .filter(|(_, val)| *val == &Some(Rock::Round))
            .map(|(point, _)| (self.grid.len_y() - point.y) as u64)
            .sum()
    }
    fn get_ranges(&self, direction: Direction) -> ([Box<dyn RangeIterator>; 2], ToPointFun) {
        match direction {
            Direction::Up => (
                [
                    Box::new(0..self.grid.len_x()),
                    Box::new(0..self.grid.len_y()),
                ],
                &|outer, inner| Point { x: outer, y: inner },
            ),
            Direction::Down => (
                [
                    Box::new(0..self.grid.len_x()),
                    Box::new((0..self.grid.len_y()).rev()),
                ],
                &|outer, inner| Point { x: outer, y: inner },
            ),
            Direction::Left => (
                [
                    Box::new(0..self.grid.len_y()),
                    Box::new(0..self.grid.len_x()),
                ],
                &|outer, inner| Point { x: inner, y: outer },
            ),
            Direction::Right => (
                [
                    Box::new(0..self.grid.len_y()),
                    Box::new((0..self.grid.len_x()).rev()),
                ],
                &|outer, inner| Point { x: inner, y: outer },
            ),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
enum Rock {
    Round,
    Square,
}

trait RangeIterator: Iterator<Item = usize> {
    fn box_clone(&self) -> Box<dyn RangeIterator>;
}
impl<T> RangeIterator for T
where
    T: Iterator<Item = usize> + Clone + 'static,
{
    fn box_clone(&self) -> Box<dyn RangeIterator> {
        Box::new(self.clone())
    }
}

type ToPointFun = &'static dyn Fn(usize, usize) -> Point<usize>;

impl FromStr for Platform {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            grid: Grid::try_from_iter(s.lines().map(|line| {
                line.chars().map(|char| match char {
                    '.' => Ok(None),
                    'O' => Ok(Some(Rock::Round)),
                    '#' => Ok(Some(Rock::Square)),
                    other => Err(anyhow::anyhow!("invalid character '{}'", other)),
                })
            }))?,
        })
    }
}
