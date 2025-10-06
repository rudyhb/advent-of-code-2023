use crate::common::day_setup::Day;
use crate::common::models::grid::GridLike;
use crate::common::models::{DirectionFlag, Grid, Point};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

pub fn day() -> Day {
    Day::new(run).with_test_inputs(&["467..114..
...*......
..35..633.
......#...
617*......
.....+.58.
..592.....
......755.
...$.*....
.664.598.."])
}
pub fn run(input: &str) {
    let schematic: Schematic = input.parse().unwrap();
    let sum = schematic.get_sum_part_numbers();
    println!("part 1 sum: {}", sum);

    let sum = schematic.get_sum_gear_ratios();
    println!("part 2 sum gear ratios: {}", sum);
}

struct Schematic {
    numbers: HashMap<Point<usize>, Number>,
    asterisks: Vec<Point<usize>>,
    grid: Grid<char>,
}

impl Schematic {
    fn number_is_part(&self, number: &Number) -> bool {
        log::trace!("starting number {:?}", number);
        let mut point = number.position;
        let mut neighbors = self.grid.neighbors(
            &point,
            DirectionFlag::LEFT
                | DirectionFlag::UP_LEFT
                | DirectionFlag::DOWN_LEFT
                | DirectionFlag::UP
                | DirectionFlag::DOWN,
        );
        for _ in 1..number.len {
            point.x += 1;
            neighbors.extend(
                self.grid
                    .neighbors(&point, DirectionFlag::UP | DirectionFlag::DOWN),
            );
        }
        neighbors.extend(self.grid.neighbors(
            &point,
            DirectionFlag::RIGHT | DirectionFlag::UP_RIGHT | DirectionFlag::DOWN_RIGHT,
        ));

        neighbors.iter().any(|neighbor| {
            let is_neighbor = self.is_symbol(neighbor);
            log::trace!(
                "position {} is symbol: '{}'={}",
                neighbor,
                self.grid[neighbor],
                is_neighbor
            );
            is_neighbor
        })
    }
    fn is_symbol(&self, point: &Point<usize>) -> bool {
        let c = self.grid[point];
        c != '.' && !c.is_numeric()
    }
    pub fn get_sum_part_numbers(&self) -> u64 {
        self.numbers
            .values()
            .filter(|number| {
                if self.number_is_part(number) {
                    log::debug!("number {} is next to a symbol", number.value);
                    true
                } else {
                    log::debug!("number {} is NOT next to a symbol", number.value);
                    false
                }
            })
            .map(|number| number.value)
            .sum()
    }
    fn get_two_adjacent_numbers(&self, point: &Point<usize>) -> Option<(u64, u64)> {
        let numbers: HashSet<_> = self
            .grid
            .neighbors(point, DirectionFlag::ALL_DIRECTIONS)
            .into_iter()
            .filter(|neighbor| self.grid[neighbor].is_numeric())
            .map(|neighbor| self.get_number_at(neighbor))
            .collect();

        if numbers.len() == 2 {
            let mut numbers = numbers.into_iter();
            Some((numbers.next().unwrap().value, numbers.next().unwrap().value))
        } else {
            None
        }
    }
    fn get_number_at(&self, mut point: Point<usize>) -> Number {
        loop {
            if let Some(number) = self.numbers.get(&point) {
                return number.clone();
            }
            point.x -= 1;
        }
    }
    pub fn get_sum_gear_ratios(&self) -> u64 {
        self.asterisks
            .iter()
            .filter_map(|asterisk| self.get_two_adjacent_numbers(asterisk))
            .map(|(left, right)| left * right)
            .sum()
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
struct Number {
    position: Point<usize>,
    len: usize,
    value: u64,
}

impl FromStr for Schematic {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let grid = Grid::from_iter(s.lines().map(|line| line.chars()));

        let mut asterisks: Vec<Point<usize>> = Default::default();
        let mut numbers: HashMap<Point<usize>, Number> = HashMap::new();
        let mut add_number = |number: Vec<char>, x: usize, y: usize| -> Result<(), anyhow::Error> {
            if number.is_empty() {
                return Ok(());
            }
            let len = number.len();
            let value = number.into_iter().collect::<String>().parse()?;
            let position = Point { x, y };
            numbers.insert(
                position,
                Number {
                    position,
                    len,
                    value,
                },
            );

            Ok(())
        };

        for (y, line) in s.lines().enumerate() {
            let mut number = vec![];
            let mut x = 0;
            for c in line.chars() {
                if c == '*' {
                    asterisks.push(Point { x, y })
                }

                if c.is_numeric() {
                    number.push(c);
                } else if x > 0 {
                    let x = x - number.len();
                    add_number(std::mem::take(&mut number), x, y)?;
                }
                x += 1;
            }
            let x = x - number.len();
            add_number(number, x, y)?;
        }

        Ok(Self {
            numbers,
            asterisks,
            grid,
        })
    }
}
