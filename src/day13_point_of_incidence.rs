use crate::common::models::{Grid, Point};
use crate::common::{Context, InputProvider};
use std::str::FromStr;

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());
    let input = context.get_input();
    let patterns: Vec<Pattern> = input
        .split("\n\n")
        .map(|chunk| chunk.parse().unwrap())
        .collect();
    let summary: usize = patterns
        .iter()
        .map(|pattern| pattern.get_split())
        .map(|split| split.summary())
        .sum();
    println!("part 1 summary: {}", summary);

    let summary: usize = patterns
        .iter()
        .map(|pattern| pattern.split_with_fixed_smudge())
        .map(|split| split.summary())
        .sum();
    println!("part 2 summary: {}", summary);
}

struct Pattern(Grid<bool>);

impl Pattern {
    pub fn split_with_fixed_smudge(&self) -> Split {
        let splits: Vec<_> = (1..self.0.len_x())
            .filter_map(|x| self.try_fix_smudge(x, false).map(|_| Split::Vertical(x)))
            .chain(
                (1..self.0.len_y())
                    .filter_map(|y| self.try_fix_smudge(y, true).map(|_| Split::Horizontal(y))),
            )
            .collect();
        self.output_split(splits)
    }
    fn output_split(&self, splits: Vec<Split>) -> Split {
        if splits.len() != 1 {
            panic!(
                "found {} splits for {}",
                splits.len(),
                self.0
                    .display_overriding(|point| { Some(if self.0[point] { '#' } else { '.' }) })
            );
        }
        splits.into_iter().next().unwrap()
    }
    pub fn get_split(&self) -> Split {
        let splits: Vec<_> = (1..self.0.len_x())
            .filter_map(|x| {
                if self.is_vertical_split(x) {
                    Some(Split::Vertical(x))
                } else {
                    None
                }
            })
            .chain((1..self.0.len_y()).filter_map(|y| {
                if self.is_horizontal_split(y) {
                    Some(Split::Horizontal(y))
                } else {
                    None
                }
            }))
            .collect();
        self.output_split(splits)
    }
    fn is_vertical_split(&self, split: usize) -> bool {
        (0..split)
            .rev()
            .zip(split..self.0.len_x())
            .all(|(left, right)| (0..self.0.len_y()).all(|y| self.0[y][left] == self.0[y][right]))
    }
    fn is_horizontal_split(&self, split: usize) -> bool {
        (0..split)
            .rev()
            .zip(split..self.0.len_y())
            .all(|(left, right)| (0..self.0.len_x()).all(|x| self.0[left][x] == self.0[right][x]))
    }
    fn try_fix_smudge(&self, split: usize, is_horizontal: bool) -> Option<Point<usize>> {
        let (max_lr, max_z) = if is_horizontal {
            (self.0.len_y(), self.0.len_x())
        } else {
            (self.0.len_x(), self.0.len_y())
        };
        let get = |left_right: usize, z: usize| -> bool {
            if is_horizontal {
                self.0[left_right][z]
            } else {
                self.0[z][left_right]
            }
        };

        let mut smudge = None;
        for (left, right) in (0..split).rev().zip(split..max_lr) {
            let mut wrong = None;
            for z in 0..max_z {
                if get(left, z) != get(right, z) {
                    if wrong.is_some() {
                        return None;
                    }
                    wrong = Some(z);
                }
            }
            if let Some(z) = wrong {
                if smudge.is_some() {
                    return None;
                }
                smudge = if is_horizontal {
                    Some(Point { x: z, y: left })
                } else {
                    Some(Point { x: left, y: z })
                };
            }
        }

        smudge
    }
}

#[derive(Debug)]
enum Split {
    Horizontal(usize),
    Vertical(usize),
}

impl Split {
    pub fn summary(&self) -> usize {
        log::debug!("split: {:?}", self);
        match self {
            Split::Horizontal(val) => 100 * *val,
            Split::Vertical(val) => *val,
        }
    }
}

impl FromStr for Pattern {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Grid::try_from_iter(s.lines().map(|line| {
            line.chars().map(|c| {
                Ok(match c {
                    '#' => true,
                    '.' => false,
                    other => return Err(anyhow::anyhow!("invalid character '{}'", other)),
                })
            })
        }))?))
    }
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    ["#.##..##.
..#.##.#.
##......#
##......#
..#.##.#.
..##..##.
#.#.##.#.

#...##..#
#....#..#
..##..###
#####.##.
#####.##.
..##..###
#....#..#"]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
