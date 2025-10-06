use crate::common::day_setup::Day;
use crate::common::models::Point;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

pub fn day() -> Day {
    Day::new(run).with_test_inputs(&["...#......
.......#..
#.........
..........
......#...
.#........
.........#
..........
.......#..
#...#....."])
}
pub fn run(input: &str) {
    let mut universe: Universe = input.parse().unwrap();
    println!("part 1 sum of lengths: {}", universe.sum_distances());
    //universe.set_expansion(10);
    universe.set_expansion(1_000_000);
    println!(
        "part 2 sum of lengths with expansion {}: {}",
        universe.get_expansion(),
        universe.sum_distances()
    );
}

struct Universe {
    x_y: HashMap<usize, HashSet<usize>>,
    y_x: HashMap<usize, HashSet<usize>>,
    expansion: usize,
}

impl Universe {
    fn new(galaxies: impl Iterator<Item = Point<usize>>) -> Self {
        let mut x_y: HashMap<usize, HashSet<usize>> = Default::default();
        let mut y_x: HashMap<usize, HashSet<usize>> = Default::default();
        for Point { x, y } in galaxies {
            x_y.entry(x).or_default().insert(y);
            y_x.entry(y).or_default().insert(x);
        }
        Self {
            x_y,
            y_x,
            expansion: 2,
        }
    }
    pub fn set_expansion(&mut self, new_expansion: usize) {
        self.expansion = new_expansion;
    }
    pub fn get_expansion(&self) -> usize {
        self.expansion
    }
    fn calculate_distance(&self, from: &Point<usize>, to: &Point<usize>) -> usize {
        from.manhattan_distance(to)
            + (from.x.min(to.x) + 1..from.x.max(to.x))
                .filter(|x| !self.x_y.contains_key(x))
                .count()
                * (self.expansion - 1)
            + (from.y.min(to.y) + 1..from.y.max(to.y))
                .filter(|y| !self.y_x.contains_key(y))
                .count()
                * (self.expansion - 1)
    }
    pub fn sum_distances(&self) -> usize {
        self.x_y
            .iter()
            .flat_map(|(&x, ys)| ys.iter().map(move |&y| Point { x, y }))
            .combinations(2)
            .map(|items| self.calculate_distance(&items[0], &items[1]))
            .sum()
    }
}

impl FromStr for Universe {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s.lines().enumerate().flat_map(|(y, line)| {
            line.chars().enumerate().filter_map(
                move |(x, c)| {
                    if c == '#' { Some(Point { x, y }) } else { None }
                },
            )
        })))
    }
}
