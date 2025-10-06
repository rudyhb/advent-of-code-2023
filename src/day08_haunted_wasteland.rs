use crate::common::day_setup::Day;
use crate::common::helpers::least_common_multiple_for;
use anyhow::Context as AnyhowContext;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::str::FromStr;

pub fn day() -> Day {
    Day::new(run).with_test_inputs(&[
        "LLR

AAA = (BBB, BBB)
BBB = (AAA, ZZZ)
ZZZ = (ZZZ, ZZZ)",
        "LR

11A = (11B, XXX)
11B = (XXX, 11Z)
11Z = (11B, XXX)
22A = (22B, XXX)
22B = (22C, 22C)
22C = (22Z, 22Z)
22Z = (22B, 22B)
XXX = (XXX, XXX)",
    ])
}

pub fn run(input: &str) {
    let network: Network = input.parse().unwrap();

    if network.nodes.contains_key("AAA") {
        let start = "AAA";
        let end = "ZZZ";
        println!(
            "part 1 - steps between {} and {}: {}",
            start,
            end,
            network.steps_between(start, end)
        );
    }

    println!(
        "part 2 - steps between all A and Z: {}",
        network.steps_for_all_a_to_z()
    );
}

struct Network {
    instructions: Vec<Direction>,
    nodes: HashMap<String, Node>,
}

impl Network {
    fn new(instructions: Vec<Direction>, nodes: impl Iterator<Item = Node>) -> Self {
        Self {
            instructions,
            nodes: nodes.map(|node| (node.name.clone(), node)).collect(),
        }
    }
    pub fn steps_between(&self, from: &str, to: &str) -> usize {
        let mut current = from;
        const MAX_ITERATIONS: usize = 1_000_000;
        for i in 0..MAX_ITERATIONS {
            if current == to {
                return i;
            }
            let direction = self.instructions[i % self.instructions.len()];
            current = self.nodes[current].get(direction);
        }
        panic!("max iterations exhausted");
    }
    pub fn steps_for_all_a_to_z(&self) -> usize {
        let mut current: Vec<_> = self
            .nodes
            .keys()
            .filter(|name| name.ends_with("A"))
            .map(|name| NodeHistory::new(name))
            .collect();
        const MAX_ITERATIONS: usize = 1_000_000;
        for i in 0..MAX_ITERATIONS {
            current
                .iter_mut()
                .for_each(|node| node.visit(i, self.instructions.len()));
            if current.iter().all(|node| node.is_done()) {
                println!("done. Getting least common multiple:\n{:#?}", current);
                return least_common_multiple_for(
                    &current
                        .iter()
                        .filter_map(|node| node.end_in_z.iter().next().map(|&val| val as u64))
                        .collect::<Vec<_>>(),
                ) as usize;
            }
            let direction = self.instructions[i % self.instructions.len()];
            for current in current.iter_mut() {
                current.next(self.nodes[current.current].get(direction))
            }
        }
        panic!("max iterations exhausted");
    }
}

struct NodeHistory<'a> {
    current: &'a str,
    visited: HashSet<(&'a str, usize)>,
    end_in_z: HashSet<usize>,
    done: bool,
}

impl<'a> NodeHistory<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            current: name,
            visited: Default::default(),
            end_in_z: Default::default(),
            done: false,
        }
    }
    pub fn is_done(&self) -> bool {
        self.done
    }
    pub fn visit(&mut self, step: usize, instruction_length: usize) {
        if self.done {
            return;
        }
        if !self
            .visited
            .insert((self.current, step % instruction_length))
        {
            self.done = true;
            return;
        }
        if self.current.ends_with("Z") {
            self.end_in_z.insert(step);
            self.done = true;
        }
    }
    pub fn next(&mut self, name: &'a str) {
        self.current = name;
    }
}

impl Debug for NodeHistory<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "done={}, end in z={:?}", self.done, self.end_in_z)
    }
}

#[derive(Copy, Clone)]
enum Direction {
    Left,
    Right,
}

struct Node {
    name: String,
    left: String,
    right: String,
}

impl Node {
    pub fn get(&self, direction: Direction) -> &str {
        match direction {
            Direction::Left => &self.left,
            Direction::Right => &self.right,
        }
    }
}

impl FromStr for Node {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        static RE: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"(?<name>[0-9A-Z]{3}) = \((?<left>[0-9A-Z]{3}), (?<right>[0-9A-Z]{3})\)")
                .unwrap()
        });
        let captures = RE.captures(s).context("invalid node format")?;

        Ok(Self {
            name: captures["name"].to_string(),
            left: captures["left"].to_string(),
            right: captures["right"].to_string(),
        })
    }
}

impl FromStr for Network {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split("\n\n");
        let instructions: Vec<Direction> = parts
            .next()
            .unwrap()
            .chars()
            .map(|c| match c {
                'R' => Ok(Direction::Right),
                'L' => Ok(Direction::Left),
                other => Err(anyhow::anyhow!("invalid direction '{}'", other)),
            })
            .collect::<Result<Vec<_>, _>>()?;

        let nodes: Vec<Node> = parts
            .next()
            .context("network missing nodes")?
            .lines()
            .map(|line| line.parse())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self::new(instructions, nodes.into_iter()))
    }
}
