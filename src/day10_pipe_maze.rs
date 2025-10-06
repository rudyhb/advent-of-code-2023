use crate::common::day_setup::Day;
use crate::common::models::grid::GridLike;
use crate::common::models::{Direction, DirectionFlag, Grid, Point};
use anyhow::Context as AnyhowContext;
use colored::Colorize;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub fn day() -> Day {
    Day::new(run).with_test_inputs(&[
        "..F7.
.FJ|.
SJ.L7
|F--J
LJ...",
        "...........
.S-------7.
.|F-----7|.
.||.....||.
.||.....||.
.|L-7.F-J|.
.|..|.|..|.
.L--J.L--J.
...........",
        ".F----7F7F7F7F-7....
.|F--7||||||||FJ....
.||.FJ||||||||L7....
FJL7L7LJLJ||LJ.L-7..
L--J.L7...LJS7F-7L7.
....F-J..F7FJ|L7L7L7
....L7.F7||L7|.L7L7|
.....|FJLJ|FJ|F7|.LJ
....FJL-7.||.||||...
....L---J.LJ.LJLJ...",
        "FF7FSF7F7F7F7F7F---7
L|LJ||||||||||||F--J
FL-7LJLJ||||||LJL-77
F--JF--7||LJLJ7F7FJ-
L---JF-JLJ.||-FJLJJ7
|F|F-JF---7F7-L7L|7|
|FFJF7L7F-JF7|JL---7
7-L-JL7||F7|L7F-7F7|
L.L7LFJ|||||FJL7||LJ
L7JLJL-JLJLJL--JLJ.L",
    ])
}
pub fn run(input: &str) {
    let map: Map = input.parse::<InputMap>().unwrap().try_into().unwrap();

    println!("part 1 - max steps: {}", map.get_loop().len() / 2);
    println!("part 2 - enclosed tiles: {}", map.get_enclosed_tiles());
}

struct Map {
    grid: Grid<Space>,
    start_position: Point<usize>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Space {
    Empty,
    HorizontalPipe,
    VerticalPipe,
    NorthEastBend,
    NorthWestBend,
    SouthWestBend,
    SouthEastBend,
}

impl Default for Space {
    fn default() -> Self {
        Self::Empty
    }
}

impl Map {
    pub fn get_loop(&self) -> Vec<Point<usize>> {
        const MAX_ITER: u64 = 10_000_000;
        let mut entered_from = self.grid[&self.start_position].move_out_directions()[0];
        let mut points = vec![self.start_position];
        for i in 1..MAX_ITER {
            let point = &points[points.len() - 1];
            let exit_to = self.grid[point].get_other_move_out_direction(entered_from);
            entered_from = exit_to.invert();
            if let Some(next) = self
                .grid
                .move_in_direction_if(point, exit_to, |(_, value)| {
                    value.can_enter_from(entered_from)
                })
            {
                points.push(next);
            } else {
                panic!("pipe closed at i = {}", i);
            }
            if points.last() == Some(&self.start_position) {
                log::debug!("looped!");
                return points;
            }
        }
        panic!("max iterations exceeded");
    }
    fn is_inside(&self, mut point: Point<usize>, part_of_loop: &HashSet<Point<usize>>) -> bool {
        let mut crossings = 0;
        while point.x < self.grid.len_x() {
            point.x += 1;
            if part_of_loop.contains(&point) {
                match self.grid[&point] {
                    Space::HorizontalPipe => {}
                    Space::VerticalPipe => {
                        crossings += 1;
                    }
                    Space::NorthEastBend | Space::NorthWestBend => {
                        // (NE + NW) and (SE + SW) cancel out - can be counted 0 or 2 times
                        // L-J F-7

                        // (NE + SW) and (SE + NW) form 1 vertical line - must be counted once
                        // L-7 F-J
                    }
                    Space::SouthWestBend | Space::SouthEastBend => {
                        crossings += 1;
                    }
                    Space::Empty => unreachable!(),
                }
            }
        }
        log::trace!("{} has {} crossings", point, crossings);
        crossings % 2 == 1
    }
    pub fn get_enclosed_tiles(&self) -> usize {
        let part_of_loop: HashSet<_> = self.get_loop().into_iter().collect();

        let inside: HashSet<_> = self
            .grid
            .iter()
            .map(|(point, _)| point)
            .filter(|point| !part_of_loop.contains(point))
            .filter(|point| self.is_inside(*point, &part_of_loop))
            .collect();

        println!(
            "grid:\n{}",
            self.grid.display_overriding(|point| {
                if &self.start_position == point {
                    Some(self.grid[point].to_string().green())
                } else if inside.contains(point) {
                    Some(self.grid[point].to_string().red())
                } else if part_of_loop.contains(point) {
                    Some(self.grid[point].to_string().blue())
                } else {
                    None
                }
            })
        );
        inside.len()
    }
}

impl Space {
    pub fn get_other_move_out_direction(&self, direction: Direction) -> Direction {
        self.move_out_directions()
            .into_iter()
            .find(|&other| other != direction)
            .with_context(|| {
                format!(
                    "inconsistent get_other_move_out_direction. input={:?} possibilities={:?}",
                    direction,
                    self.move_out_directions()
                )
            })
            .unwrap()
    }
    pub fn move_out_directions(&self) -> [Direction; 2] {
        match self {
            Space::Empty => {
                panic!("invalid move directions call")
            }
            Space::HorizontalPipe => [Direction::Left, Direction::Right],
            Space::VerticalPipe => [Direction::Up, Direction::Down],
            Space::NorthEastBend => [Direction::Up, Direction::Right],
            Space::NorthWestBend => [Direction::Up, Direction::Left],
            Space::SouthWestBend => [Direction::Down, Direction::Left],
            Space::SouthEastBend => [Direction::Down, Direction::Right],
        }
    }
    pub fn can_enter_from(&self, direction: Direction) -> bool {
        match self {
            Space::Empty => false,
            Space::HorizontalPipe => {
                matches!(direction, Direction::Left | Direction::Right)
            }
            Space::VerticalPipe => {
                matches!(direction, Direction::Up | Direction::Down)
            }
            Space::NorthEastBend => {
                matches!(direction, Direction::Up | Direction::Right)
            }
            Space::NorthWestBend => {
                matches!(direction, Direction::Up | Direction::Left)
            }
            Space::SouthWestBend => {
                matches!(direction, Direction::Down | Direction::Left)
            }
            Space::SouthEastBend => {
                matches!(direction, Direction::Down | Direction::Right)
            }
        }
    }
    pub fn all_spaces() -> &'static [Self] {
        use Space::*;
        &[
            Empty,
            HorizontalPipe,
            VerticalPipe,
            NorthEastBend,
            NorthWestBend,
            SouthWestBend,
            SouthEastBend,
        ]
    }
}

impl TryFrom<InputMap> for Map {
    type Error = anyhow::Error;

    fn try_from(mut value: InputMap) -> Result<Self, Self::Error> {
        let (start_position, _) = value
            .0
            .iter()
            .find(|(_, space)| matches!(space, InputSpace::Start))
            .context("cannot find starting space")?;
        let space_type = detect_space_type(&start_position, &value)?;
        value.0[&start_position] = InputSpace::Space(space_type);

        Ok(Self {
            grid: Grid::try_from_iter(value.0.iter_rows().map(|(_, row)| {
                row.map(|(_, space)| {
                    if let InputSpace::Space(space) = space {
                        Ok(*space)
                    } else {
                        Err(anyhow::anyhow!("should not have starting position anymore"))
                    }
                })
            }))?,
            start_position,
        })
    }
}

fn detect_space_type(point: &Point<usize>, map: &InputMap) -> anyhow::Result<Space> {
    let neighbor_directions: Vec<_> = map
        .0
        .neighbors(point, DirectionFlag::FOUR_DIRECTIONS)
        .into_iter()
        .filter_map(|neighbor| {
            let direction = Direction::from_vec(&neighbor, point).unwrap();
            if let InputSpace::Space(space) = &map.0[&neighbor] {
                if space.can_enter_from(direction) {
                    return Some(direction.invert());
                }
            }
            None
        })
        .collect();
    if neighbor_directions.len() != 2 {
        return Err(anyhow::anyhow!(
            "starting position has {} possible neighbors",
            neighbor_directions.len()
        ));
    }

    let space_type: Vec<_> = Space::all_spaces()
        .iter()
        .copied()
        .filter(|space| {
            space.can_enter_from(neighbor_directions[0])
                && space.can_enter_from(neighbor_directions[1])
        })
        .collect();
    if space_type.len() != 1 {
        return Err(anyhow::anyhow!(
            "starting position has {} possible types...",
            space_type.len()
        ));
    }
    log::debug!("starting space is {:?}", space_type[0]);
    Ok(space_type[0])
}

enum InputSpace {
    Start,
    Space(Space),
}

struct InputMap(Grid<InputSpace>);

impl FromStr for InputMap {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Space::*;
        Ok(Self(Grid::try_from_iter(s.lines().map(|line| {
            line.chars().map(|c| {
                Ok(InputSpace::Space(match c {
                    'S' => {
                        return Ok(InputSpace::Start);
                    }
                    '|' => VerticalPipe,
                    '-' => HorizontalPipe,
                    'L' => NorthEastBend,
                    'J' => NorthWestBend,
                    '7' => SouthWestBend,
                    'F' => SouthEastBend,
                    '.' => Empty,
                    other => return Err(anyhow::anyhow!("invalid character '{}'", other)),
                }))
            })
        }))?))
    }
}

impl Display for Space {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Space::Empty => write!(f, "."),
            Space::HorizontalPipe => write!(f, "-"),
            Space::VerticalPipe => write!(f, "|"),
            Space::NorthEastBend => write!(f, "L"),
            Space::NorthWestBend => write!(f, "J"),
            Space::SouthWestBend => write!(f, "7"),
            Space::SouthEastBend => write!(f, "F"),
        }
    }
}
