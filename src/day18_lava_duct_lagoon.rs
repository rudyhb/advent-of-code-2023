use crate::common::day_setup::Day;
use crate::common::models::{Direction, Point};
use anyhow::Context as AnyhowContext;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::ops::RangeInclusive;
use std::str::FromStr;

pub fn day() -> Day {
    Day::new(run).with_test_inputs(&["R 6 (#70c710)
D 5 (#0dc571)
L 2 (#5713f0)
D 2 (#d2c081)
R 2 (#59c680)
D 2 (#411b91)
L 5 (#8ceee2)
U 2 (#caa173)
L 1 (#1b58a2)
U 2 (#caa171)
R 2 (#7807d2)
U 3 (#a77fa3)
L 2 (#015232)
U 2 (#7a21e3)"])
}
pub fn run(input: &str) {
    let instructions: Vec<Instruction> = input.lines().map(|line| line.parse().unwrap()).collect();

    let lagoon = Lagoon::dig_edges(&instructions);
    log::debug!("{}", lagoon);
    println!("part 1: {} m3", lagoon.count_inside());

    let instructions: Vec<CorrectedInstruction> = instructions
        .into_iter()
        .map(|i| CorrectedInstruction {
            direction: i.direction,
            count: i.count as i64,
        })
        .collect();

    let lagoon = LagoonV2::build(&instructions);
    println!("part 1 v2: {} m3", lagoon.count_dug());

    let instructions: Vec<CorrectedInstruction> =
        input.lines().map(|line| line.parse().unwrap()).collect();

    let lagoon = LagoonV2::build(&instructions);
    println!("part 2: {} m3", lagoon.count_dug());
}

struct LagoonV2 {
    vertical_lines: HashMap<i64, BTreeMap<i64, i64>>,
    horizontal_lines: HashMap<i64, BTreeMap<i64, i64>>,
    x_s: BTreeSet<i64>,
    y_s: BTreeSet<i64>,
}

impl LagoonV2 {
    pub fn build(instructions: &[CorrectedInstruction]) -> Self {
        let mut point = Point { x: 0i64, y: 0 };
        let mut vertical_lines: HashMap<i64, BTreeMap<i64, i64>> = Default::default();
        let mut horizontal_lines: HashMap<i64, BTreeMap<i64, i64>> = Default::default();
        let mut x_s = BTreeSet::default();
        let mut y_s = BTreeSet::default();
        let mut update = |point: &Point<i64>| {
            x_s.insert(point.x);
            y_s.insert(point.y);
        };
        update(&point);

        for CorrectedInstruction { direction, count } in instructions {
            match *direction {
                Direction::Up => {
                    vertical_lines
                        .entry(point.x)
                        .or_default()
                        .insert(point.y - *count, point.y);
                    point.y -= *count;
                }
                Direction::Down => {
                    vertical_lines
                        .entry(point.x)
                        .or_default()
                        .insert(point.y, point.y + *count);
                    point.y += *count;
                }
                Direction::Left => {
                    horizontal_lines
                        .entry(point.y)
                        .or_default()
                        .insert(point.x - *count, point.x);
                    point.x -= *count;
                }
                Direction::Right => {
                    horizontal_lines
                        .entry(point.y)
                        .or_default()
                        .insert(point.x, point.x + *count);
                    point.x += *count;
                }
            }
            update(&point);
        }

        assert_eq!(point, Point::default(), "not a closed loop");
        Self {
            vertical_lines,
            horizontal_lines,
            x_s,
            y_s,
        }
    }
    pub fn count_dug(&self) -> i64 {
        let mut areas = BlockAreas::default();

        // numerically integrating for the area
        for &y in self.y_s.iter().take(self.y_s.len() - 1) {
            areas.next_line(y, &self.horizontal_lines);
            for &x in self.x_s.iter() {
                areas.try_next_x(x, &self.vertical_lines);
            }
        }

        areas.next_line(*self.y_s.last().unwrap(), &self.horizontal_lines);

        areas.total_count + self.count_dug_edges()
    }
    fn count_dug_edges(&self) -> i64 {
        self.horizontal_lines
            .values()
            .chain(self.vertical_lines.values())
            .flatten()
            .map(|(&start, &end)| end - start)
            .sum()
    }
}

#[derive(Default)]
struct BlockAreas {
    total_count: i64,
    current_blocks: Vec<(i64, i64)>,
    current_y: i64,
    current_start_x: Option<i64>,
}

impl BlockAreas {
    pub fn next_line(&mut self, next_y: i64, horizontal_lines: &HashMap<i64, BTreeMap<i64, i64>>) {
        if self.current_start_x.is_some() {
            panic!("left a dangling start_x at line {}", self.current_y);
        }

        let horizontal = horizontal_lines.get(&self.current_y);

        for (start_x, end_x) in std::mem::take(&mut self.current_blocks) {
            let mut sum = (end_x - start_x - 1) * (next_y - self.current_y);
            if let Some(horizontal) = horizontal {
                // don't double count edges
                sum -= Self::get_intersection(start_x + 1..=end_x - 1, horizontal);
            }
            log::debug!(
                "adding area x ({} -> {}) y ({} -> {}) = {}",
                start_x,
                end_x,
                self.current_y,
                next_y,
                sum
            );
            self.total_count += sum;
        }
        self.current_y = next_y;
    }
    pub fn try_next_x(&mut self, x: i64, vertical_lines: &HashMap<i64, BTreeMap<i64, i64>>) {
        if let Some(lines_at_x) = vertical_lines.get(&x) {
            if let Some((_closest_back, &closest_front)) =
                lines_at_x.range(..=self.current_y).next_back()
            {
                if closest_front > self.current_y {
                    // has to be strictly greater
                    self.set_next_x(x);
                }
            }
        }
    }
    fn set_next_x(&mut self, x: i64) {
        if let Some(start_x) = self.current_start_x {
            self.current_blocks.push((start_x, x));
            self.current_start_x = None;
        } else {
            self.current_start_x = Some(x);
        }
    }
    fn get_intersection(range: RangeInclusive<i64>, lines: &BTreeMap<i64, i64>) -> i64 {
        let mut intersection = 0i64;
        if let Some((_start, &end)) = lines.range(..*range.start()).next_back() {
            if end >= *range.start() {
                intersection += (*range.end()).min(end) - *range.start() + 1;
            }
        }
        for (&start, &end) in lines.range(*range.start()..=*range.end()) {
            intersection += (*range.end()).min(end) - start + 1;
        }

        intersection
    }
}

struct Lagoon {
    dug: HashSet<Point<i64>>,
    range_x: RangeInclusive<i64>,
    range_y: RangeInclusive<i64>,
}

impl Lagoon {
    pub fn count_inside(&self) -> usize {
        self.range_y
            .clone()
            .flat_map(|y| self.range_x.clone().map(move |x| Point { x, y }))
            .filter(|point| self.dug.contains(point) || self.count_edge_crossings(*point) % 2 == 1)
            .count()
    }
    fn count_edge_crossings(&self, mut point: Point<i64>) -> usize {
        let mut count = 0;
        let mut on_edge = false;
        let mut on_left = false;
        let mut on_right = false;
        let mut set_on = |point: &Point<i64>| {
            let current_on = self.dug.contains(point);
            if current_on {
                on_edge = true;
                if !on_left {
                    on_left = self
                        .dug
                        .contains(&point.move_in_direction_unchecked(Direction::Up));
                }
                if !on_right {
                    on_right = self
                        .dug
                        .contains(&point.move_in_direction_unchecked(Direction::Down));
                }
            } else if on_edge {
                if on_left && on_right {
                    count += 1;
                }

                on_edge = false;
                on_left = false;
                on_right = false;
            }
        };

        while point.x < self.range_x.end() + 2 {
            point.x += 1;
            set_on(&point);
        }

        count
    }
    fn new(edges: HashSet<Point<i64>>) -> Self {
        let range_x =
            edges.iter().map(|p| p.x).min().unwrap()..=edges.iter().map(|p| p.x).max().unwrap();
        let range_y =
            edges.iter().map(|p| p.y).min().unwrap()..=edges.iter().map(|p| p.y).max().unwrap();

        Self {
            dug: edges,
            range_x,
            range_y,
        }
    }
    pub fn dig_edges(instructions: &[Instruction]) -> Self {
        let mut point = Point { x: 0i64, y: 0 };
        let mut set = HashSet::new();

        for Instruction {
            direction, count, ..
        } in instructions
        {
            for _ in 0..*count {
                point = point.move_in_direction_unchecked(*direction);
                set.insert(point);
            }
        }

        assert_eq!(point, Point::default(), "not a closed loop");
        Self::new(set)
    }
}

struct Instruction {
    direction: Direction,
    count: usize,
    #[allow(dead_code)]
    color: Rgb,
}

struct CorrectedInstruction {
    direction: Direction,
    count: i64,
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

impl Default for Rgb {
    fn default() -> Self {
        Self {
            r: 255,
            g: 255,
            b: 255,
        }
    }
}

impl FromStr for Instruction {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();
        let direction: Direction = parts.next().context("no direction")?.parse()?;
        let count: usize = parts.next().context("no size")?.parse()?;

        static RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"(\(#(?<r>[\da-f]{2})(?<g>[\da-f]{2})(?<b>[\da-f]{2})\))").unwrap()
        });

        let matches = RE
            .captures(parts.next().context("no color")?)
            .context("invalid color")?;
        let color = Rgb {
            r: u8::from_str_radix(&matches["r"], 16)?,
            g: u8::from_str_radix(&matches["g"], 16)?,
            b: u8::from_str_radix(&matches["b"], 16)?,
        };
        Ok(Self {
            direction,
            count,
            color,
        })
    }
}

impl FromStr for CorrectedInstruction {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();
        static RE: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(\(#(?<count>[\da-f]{5})(?<direction>[\da-f])\))").unwrap());

        let matches = RE
            .captures(parts.nth(2).context("no color")?)
            .context("invalid color")?;
        let direction = match &matches["direction"] {
            "0" => Direction::Right,
            "1" => Direction::Down,
            "2" => Direction::Left,
            "3" => Direction::Up,
            other => return Err(anyhow::anyhow!("invalid direction '{}'", other)),
        };
        let count = i64::from_str_radix(&matches["count"], 16)?;
        Ok(Self { direction, count })
    }
}

impl Display for Lagoon {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for y in self.range_y.clone() {
            writeln!(f)?;
            for x in self.range_x.clone() {
                let point = Point { x, y };
                if self.dug.contains(&point) {
                    write!(f, "#")?;
                } else {
                    write!(f, ".")?;
                }
            }
        }
        Ok(())
    }
}
