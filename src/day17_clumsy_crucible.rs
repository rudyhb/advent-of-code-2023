use crate::common::models::grid::GridLike;
use crate::common::models::{Direction, Grid, Point};
use crate::common::{Context, InputProvider};
use anyhow::Context as AnyhowContext;
use colored::Colorize;
use std::collections::HashMap;
use std::str::FromStr;
use utils::a_star;
use utils::a_star::{NodeSuccessorConverter, Successor};

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());
    let input = context.get_input();

    let map: CityMap = input.parse().unwrap();

    let cost = map.min_heat_loss(Crucible::new(Point::default(), Direction::Right));
    println!("part 1 min heat loss: {}", cost);

    let cost = map.min_heat_loss(UltraCrucible(Crucible::new(
        Point::default(),
        Direction::Right,
    )));
    println!("part 2 min heat loss: {}", cost);
}

struct CityMap {
    grid: Grid<u8>,
}

impl CityMap {
    pub fn min_heat_loss(&self, start: impl CrucibleLike) -> u64 {
        let end = Point {
            x: self.grid.len_x() - 1,
            y: self.grid.len_y() - 1,
        };
        let result = a_star::a_star_search(
            start,
            |current| current.next(&self.grid),
            |details| details.current_node.position().manhattan_distance(&end) as u64,
            |current| current.position() == &end,
            None,
        )
        .unwrap();
        let paths: HashMap<Point<usize>, Direction> = result
            .shortest_path
            .iter()
            .skip(1)
            .map(|point| (point.position().clone(), point.direction()))
            .collect();
        println!(
            "shortest path:{}",
            self.grid.display_overriding(|point| paths
                .get(point)
                .map(|val| val.to_string().bright_blue()))
        );
        result.shortest_path_cost
    }
}

trait CrucibleLike: a_star::Node + Sized {
    fn next(&self, grid: &Grid<u8>) -> Vec<Successor<Self, u64>>;
    fn position(&self) -> &Point<usize>;
    fn direction(&self) -> Direction;
}

#[derive(Eq, PartialEq, Hash, Debug)]
struct Crucible {
    position: Point<usize>,
    direction: Direction,
    forward_times: u8,
}

impl Crucible {
    pub fn new(position: Point<usize>, direction: Direction) -> Self {
        Self {
            position,
            direction,
            forward_times: 0,
        }
    }
}

impl CrucibleLike for Crucible {
    fn next(&self, grid: &Grid<u8>) -> Vec<Successor<Self, u64>> {
        let mut next_nodes = vec![];
        if self.forward_times < 3 {
            if let Some(next) = grid.move_in_direction_if(&self.position, self.direction, |_| true)
            {
                let cost = grid[&next];
                next_nodes.push(
                    Self {
                        position: next,
                        direction: self.direction,
                        forward_times: self.forward_times + 1,
                    }
                    .to_successor(cost as u64),
                );
            }
        }
        for direction in [self.direction.turn_left(), self.direction.turn_right()] {
            if let Some(next) = grid.move_in_direction_if(&self.position, direction, |_| true) {
                let cost = grid[&next];
                next_nodes.push(
                    Self {
                        position: next,
                        direction,
                        forward_times: 1,
                    }
                    .to_successor(cost as u64),
                );
            }
        }

        next_nodes
    }

    #[inline]
    fn position(&self) -> &Point<usize> {
        &self.position
    }

    #[inline]
    fn direction(&self) -> Direction {
        self.direction
    }
}

impl a_star::Node for Crucible {}

#[derive(Eq, PartialEq, Hash, Debug)]
struct UltraCrucible(Crucible);

impl a_star::Node for UltraCrucible {}

impl CrucibleLike for UltraCrucible {
    fn next(&self, grid: &Grid<u8>) -> Vec<Successor<Self, u64>> {
        let mut next_nodes = vec![];

        let mut try_push_next = |direction: Direction, times: u8| {
            let mut position = self.0.position.clone();
            let mut cost = 0u64;
            for _ in 0..times {
                position = if let Some(position) = position.move_in(direction) {
                    position
                } else {
                    return;
                };
                cost += if let Some(cost) = grid.get(&position) {
                    *cost as u64
                } else {
                    return;
                }
            }
            next_nodes.push(
                Self(Crucible {
                    position,
                    direction,
                    forward_times: if self.0.direction == direction {
                        self.0.forward_times + times
                    } else {
                        times
                    },
                })
                .to_successor(cost),
            );
        };

        if (4..10).contains(&self.0.forward_times) {
            try_push_next(self.0.direction, 1);
        } else if self.0.forward_times == 0 {
            try_push_next(self.0.direction, 4);
        }
        try_push_next(self.0.direction.turn_left(), 4);
        try_push_next(self.0.direction.turn_right(), 4);

        next_nodes
    }

    #[inline]
    fn position(&self) -> &Point<usize> {
        &self.0.position
    }
    #[inline]
    fn direction(&self) -> Direction {
        self.0.direction
    }
}

impl FromStr for CityMap {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            grid: Grid::try_from_iter(s.lines().map(|line| {
                line.chars()
                    .map(|c| c.to_digit(10).context("invalid number").map(|d| d as u8))
            }))?,
        })
    }
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    ["2413432311323
3215453535623
3255245654254
3446585845452
4546657867536
1438598798454
4457876987766
3637877979653
4654967986887
4564679986453
1224686865563
2546548887735
4322674655533"]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
