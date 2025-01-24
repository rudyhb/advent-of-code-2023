use crate::common::models::{Direction, Grid, Point};
use crate::common::{Context, InputProvider};
use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());
    let input = context.get_input();

    let contraption: Contraption = input.parse().unwrap();
    println!(
        "part 1 energized count: {}",
        contraption.energized_count_from(BeamPoint::new(Point::default(), Direction::Right))
    );

    optimize_energized(&contraption);
}

fn optimize_energized(contraption: &Contraption) {
    let max: usize = [
        (
            Direction::Right,
            (0..contraption.grid.len_y())
                .map(|y| Point { x: 0, y })
                .collect::<Vec<_>>(),
        ),
        (
            Direction::Down,
            (0..contraption.grid.len_x())
                .map(|x| Point { x, y: 0 })
                .collect(),
        ),
        (
            Direction::Left,
            (0..contraption.grid.len_y())
                .map(|y| Point {
                    x: contraption.grid.len_x() - 1,
                    y,
                })
                .collect(),
        ),
        (
            Direction::Up,
            (0..contraption.grid.len_x())
                .map(|x| Point {
                    x,
                    y: contraption.grid.len_y() - 1,
                })
                .collect(),
        ),
    ]
    .into_iter()
    .flat_map(|(direction, points)| {
        points
            .into_iter()
            .map(move |start| BeamPoint::new(start, direction))
    })
    .map(|start| contraption.energized_count_from(start))
    .max()
    .unwrap();
    println!("part 2 max energized count: {}", max);
}

struct Contraption {
    grid: Grid<Space>,
}

impl Contraption {
    pub fn energized_count_from(&self, start: BeamPoint) -> usize {
        let mut cache = SimpleCache::default();
        self.fill_out_energized_from(start.clone(), &mut cache);
        log::debug!(
            "energized:\n{}",
            self.grid.display_with_rule(|(point, _)| {
                if cache.cache.contains(point) {
                    "#".blue()
                } else {
                    ".".white()
                }
            })
        );
        cache.cache.len()
    }
    fn fill_out_energized_from(&self, start: BeamPoint, cache: &mut SimpleCache) {
        let mut beams = vec![BeamPath::new(vec![start])];
        while !beams.is_empty() {
            beams = beams
                .into_iter()
                .filter_map(|beam| {
                    let point = beam.points.iter().last().unwrap();
                    if cache.visited(point) {
                        return None;
                    }
                    cache.visit(point.clone());
                    if beam.is_loop().is_some() {
                        return None;
                    }
                    let next: Vec<_> = self.enter(point).collect();
                    match next.len() {
                        0 => None,
                        1 => {
                            let mut path = beam;
                            path.push(next.into_iter().next().unwrap());
                            Some(vec![path])
                        }
                        _ => Some(
                            next.into_iter()
                                .map(move |next| {
                                    let mut path = beam.clone();
                                    path.push(next);
                                    path
                                })
                                .collect(),
                        ),
                    }
                })
                .flatten()
                .collect();
        }
    }
    fn enter(&self, beam: &BeamPoint) -> impl Iterator<Item = BeamPoint> {
        let (first, second) = self.grid[&beam.location].redirect(beam.direction);
        [
            self.grid
                .move_in_direction_if(&beam.location, first, |_| true)
                .map(|point| BeamPoint::new(point, first)),
            second.and_then(|second| {
                self.grid
                    .move_in_direction_if(&beam.location, second, |_| true)
                    .map(|point| BeamPoint::new(point, second))
            }),
        ]
        .into_iter()
        .flatten()
    }
}

#[derive(Default)]
struct SimpleCache {
    cache: HashSet<Point<usize>>,
    visited: HashSet<BeamPoint>,
}

impl SimpleCache {
    pub fn visited(&self, point: &BeamPoint) -> bool {
        self.visited.contains(point)
    }
    pub fn visit(&mut self, point: BeamPoint) {
        self.cache.insert(point.location.clone());
        self.visited.insert(point);
    }
}
#[derive(Default, Clone, Debug)]
struct BeamPath {
    points: Vec<BeamPoint>,
    indexed: HashMap<BeamPoint, u8>,
}

impl BeamPath {
    pub fn new(points: Vec<BeamPoint>) -> Self {
        let indexed = points.iter().map(|point| (point.clone(), 1)).collect();
        Self { points, indexed }
    }
    pub fn is_loop(&self) -> Option<(&[BeamPoint], &[BeamPoint])> {
        let value = self.points.last().unwrap();
        if self.indexed.get(value).unwrap() > &1 {
            let end_non_inclusive = self.points.len() - 1;
            let start = self.points.iter().position(|point| point == value).unwrap();
            if start == end_non_inclusive {
                panic!("invalid loop...");
            }
            let the_loop = &self.points[start..end_non_inclusive];
            let before_loop = &self.points[0..start];
            return Some((the_loop, before_loop));
        }
        None
    }
    pub fn push(&mut self, next: BeamPoint) {
        *self.indexed.entry(next.clone()).or_default() += 1;
        self.points.push(next);
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
struct BeamPoint {
    location: Point<usize>,
    direction: Direction,
}

impl BeamPoint {
    pub fn new(location: Point<usize>, direction: Direction) -> Self {
        Self {
            location,
            direction,
        }
    }
}

#[derive(strum_macros::Display, Copy, Clone, Eq, PartialEq)]
enum Space {
    #[strum(serialize = ".")]
    Empty,
    #[strum(serialize = "/")]
    MirrorFront,
    #[strum(serialize = r"\")]
    MirrorBack,
    #[strum(serialize = "-")]
    SplitterHorizontal,
    #[strum(serialize = "|")]
    SplitterVertical,
}

impl Space {
    pub fn redirect(&self, incoming: Direction) -> (Direction, Option<Direction>) {
        match self {
            Space::Empty => (incoming, None),
            Space::MirrorFront => match incoming {
                Direction::Up => (Direction::Right, None),
                Direction::Down => (Direction::Left, None),
                Direction::Left => (Direction::Down, None),
                Direction::Right => (Direction::Up, None),
            },
            Space::MirrorBack => match incoming {
                Direction::Up => (Direction::Left, None),
                Direction::Down => (Direction::Right, None),
                Direction::Left => (Direction::Up, None),
                Direction::Right => (Direction::Down, None),
            },
            Space::SplitterHorizontal => match incoming {
                Direction::Up | Direction::Down => (Direction::Left, Some(Direction::Right)),
                Direction::Left | Direction::Right => (incoming, None),
            },
            Space::SplitterVertical => match incoming {
                Direction::Up | Direction::Down => (incoming, None),
                Direction::Left | Direction::Right => (Direction::Up, Some(Direction::Down)),
            },
        }
    }
}

impl FromStr for Contraption {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            grid: Grid::try_from_iter(s.lines().map(|line| {
                line.chars().map(|c| {
                    Ok(match c {
                        '.' => Space::Empty,
                        '/' => Space::MirrorFront,
                        '\\' => Space::MirrorBack,
                        '|' => Space::SplitterVertical,
                        '-' => Space::SplitterHorizontal,
                        other => return Err(anyhow::anyhow!("invalid character '{}'", other)),
                    })
                })
            }))?,
        })
    }
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    [r".|...\....
|.-.\.....
.....|-...
........|.
..........
.........\
..../.\\..
.-.-/..|..
.|....-|.\
..//.|...."]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
