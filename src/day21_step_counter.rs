use crate::common::day_setup::{AppContext, Day};
use crate::common::models::grid::GridLike;
use crate::common::models::{Direction, Grid, Point};
use colored::Colorize;
use std::collections::HashSet;

pub fn day() -> Day {
    Day::custom(run).with_test_inputs(&["...........
.....###.#.
.###.##..#.
..#.#...#..
....#.#....
.##..S####.
.##..#...#.
.......##..
.##.#.####.
.##..##.##.
..........."])
}
pub fn run(context: &AppContext) {
    let input = context.get_input();
    let steps: u64 = if context.is_testing() { 6 } else { 64 };

    let (garden, start_location) = parse(&input);

    let end_positions = garden.solve_for(start_location, steps);
    println!(
        "{}",
        garden.0.display_overriding(|point| {
            if end_positions.contains(point) {
                Some("O".blue())
            } else {
                None
            }
        })
    );
    println!("part 1 visited in {} steps: {}", steps, end_positions.len());

    if context.is_testing() {
        println!("part 2 only works for full version");
        return;
    }
    println!("part 2:");
    let steps = 26501365;
    // always looks diamond shape so we can probably solve geometrically
    let sol = solve_geometrically(steps, &garden);
    println!("in {} steps: {}", steps, sol);
}

fn solve_geometrically(steps: u64, garden: &Garden) -> u64 {
    assert!(
        garden
            .0
            .iter()
            .filter(|(point, _)| point.x == 0 || point.y == 0)
            .all(|(_, &space)| space == Space::Plot),
        "only works in the case that the horizontal and vertical axes are empty. Solution looks like a diamond"
    );

    let half_len = garden.half_len() as u64;
    let grid_size = half_len * 2 + 1;

    let n = (steps - half_len) / grid_size;
    assert_eq!(
        steps - half_len,
        n * grid_size,
        "method only works on exact multiples of steps"
    );

    // in the middle of the diamond
    let same_parity_full = (n + 1).pow(2);
    let other_parity_full = n.pow(2);

    // at the edges of the diamond
    let same_parity_outer_corners = n + 1; // these are outside the diamond and need to be subtracted
    let other_parity_outer_corners = n; // these are inside the diamond

    let visited_plots = garden.get_all_visited_plots(Point {
        x: half_len as usize,
        y: half_len as usize,
    });
    let visited_corner_plots: HashSet<_> = garden
        .filter_corner_points(visited_plots.iter().cloned())
        .collect();

    let visited_with_parity = |is_odd: bool, plots: &HashSet<Point<usize>>| {
        plots
            .iter()
            .filter(|point| {
                if is_odd {
                    point.parity_xor()
                } else {
                    !point.parity_xor()
                }
            })
            .count() as u64
    };

    same_parity_full * visited_with_parity(!steps.is_multiple_of(2), &visited_plots)
        + other_parity_full * visited_with_parity(steps.is_multiple_of(2), &visited_plots)
        + other_parity_outer_corners
            * visited_with_parity(steps.is_multiple_of(2), &visited_corner_plots)
        - same_parity_outer_corners
            * visited_with_parity(!steps.is_multiple_of(2), &visited_corner_plots)
}

struct Garden(Grid<Space>);
impl Garden {
    pub fn get_all_visited_plots(&self, start: Point<usize>) -> HashSet<Point<usize>> {
        let mut visited = HashSet::new();
        let mut positions = HashSet::from([start]);
        loop {
            for point in std::mem::take(&mut positions) {
                if visited.contains(&point) {
                    continue;
                }

                for next in self.get_next_positions(&point) {
                    positions.insert(next);
                }
                visited.insert(point);
            }

            if positions.is_empty() {
                break;
            }
        }

        visited
    }
    pub fn solve_for(&self, start: Point<usize>, steps: u64) -> HashSet<Point<usize>> {
        let mut positions = HashSet::from([start]);
        let mut last = positions.clone();
        for i in 1..=steps {
            for point in std::mem::take(&mut positions).into_iter() {
                for next in self.get_next_positions(&point) {
                    positions.insert(next);
                }
            }
            log::trace!(
                "step {}:{}",
                i,
                self.0.display_overriding(|point| {
                    if positions.contains(point) {
                        Some("O".blue())
                    } else if last.contains(point) {
                        Some("O".yellow())
                    } else {
                        None
                    }
                })
            );

            last = positions.clone();
        }

        positions
    }
    fn get_next_positions<'a, 'b>(
        &'a self,
        point: &'b Point<usize>,
    ) -> impl Iterator<Item = Point<usize>> + use<'a, 'b> {
        Direction::all().into_iter().filter_map(move |direction| {
            self.0
                .move_in_direction_if(point, direction, |(_, &space)| space == Space::Plot)
        })
    }
    pub fn new(grid: Grid<Space>) -> Self {
        Self(grid)
    }
    pub fn half_len(&self) -> usize {
        self.0.len_x() / 2
    }
    pub fn filter_corner_points<T: Iterator<Item = Point<usize>>>(
        &self,
        points: T,
    ) -> impl Iterator<Item = Point<usize>> + use<'_, T> {
        points.filter(|point| self.is_corner_point(&self.map_point_to_midpoint(point)))
    }

    fn map_point_to_midpoint(&self, point: &Point<usize>) -> Point<i64> {
        Point {
            x: point.x as i64 - self.half_len() as i64,
            y: point.y as i64 - self.half_len() as i64,
        }
    }

    fn is_corner_point(&self, point_centered_at_midpoint: &Point<i64>) -> bool {
        point_centered_at_midpoint.x.abs() + point_centered_at_midpoint.y.abs()
            > self.half_len() as i64
    }
}

#[derive(Default, Copy, Clone, Eq, PartialEq, strum_macros::Display)]
enum Space {
    #[default]
    #[strum(serialize = ".")]
    Plot,
    #[strum(serialize = "#")]
    Rock,
}

fn parse(input: &str) -> (Garden, Point<usize>) {
    let len_x = input
        .lines()
        .next()
        .map(|line| line.chars().count())
        .unwrap_or_default();
    let len_y = input.lines().count();
    assert_eq!(len_x, len_y);
    let half_len = (len_x / 2) as i64;
    assert_eq!(half_len * 2 + 1, len_x as i64);
    let mut grid: Grid<Space> = Grid::new(len_x, len_y);
    let mut start = None;

    for (y, line) in input.lines().enumerate() {
        for (x, c) in line.chars().enumerate() {
            let point = Point { x, y };
            match c {
                '.' => {}
                '#' => grid[&point] = Space::Rock,
                'S' => start = Some(point),
                other => panic!("invalid character '{}'", other),
            }
        }
    }

    let start = start.expect("start position not found");

    (Garden::new(grid), start)
}

struct PointConverter<'a>(&'a Point<usize>);

impl<'a> From<PointConverter<'a>> for Point<i64> {
    fn from(value: PointConverter<'a>) -> Self {
        Self {
            x: value.0.x as i64,
            y: value.0.y as i64,
        }
    }
}

trait ParityXor {
    fn parity_xor(&self) -> bool;
}

impl ParityXor for Point<usize> {
    fn parity_xor(&self) -> bool {
        (self.x % 2 == 1) != (self.y % 2 == 1)
    }
}
