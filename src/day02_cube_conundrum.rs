use crate::common::{Context, InputProvider};
use anyhow::Context as AnyhowContext;
use std::str::FromStr;

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let input = context.get_input();
    let input = input.as_str();

    let games: Vec<Game> = input.lines().map(|line| line.parse().unwrap()).collect();
    let set = Set {
        red: 12,
        green: 13,
        blue: 14,
    };
    let sum: usize = games
        .iter()
        .filter(|game| game.is_possible(&set))
        .map(|game| game.id)
        .sum();
    println!("part 1 sum: {}", sum);

    let sum: usize = games
        .iter()
        .map(|game| {
            let min_set = game.get_min_set();
            min_set.get_power()
        })
        .sum();
    println!("part 2 sum of powers: {}", sum);
}

struct Game {
    id: usize,
    sets: Vec<Set>,
}

#[derive(Default)]
struct Set {
    red: usize,
    blue: usize,
    green: usize,
}

impl Set {
    pub fn contains(&self, other: &Self) -> bool {
        self.red >= other.red && self.blue >= other.blue && self.green >= other.green
    }
    pub fn get_power(&self) -> usize {
        self.red * self.blue * self.green
    }
    pub fn expand_to(&mut self, other: &Self) {
        self.red = self.red.max(other.red);
        self.blue = self.blue.max(other.blue);
        self.green = self.green.max(other.green);
    }
}

impl Game {
    pub fn is_possible(&self, with_set: &Set) -> bool {
        self.sets.iter().all(|set| with_set.contains(set))
    }
    pub fn get_min_set(&self) -> Set {
        self.sets.iter().fold(Set::default(), |mut acc, next| {
            acc.expand_to(next);
            acc
        })
    }
}

impl FromStr for Game {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(":");
        let id: usize = parts
            .next()
            .unwrap()
            .split_whitespace()
            .last()
            .context("empty game number")?
            .parse()?;

        let sets: Vec<_> = parts
            .next()
            .context("no game body")?
            .split(";")
            .map(|set_str| {
                let mut set = Set::default();
                for part in set_str.trim().split(",") {
                    let mut parts = part.split_whitespace();
                    let count: usize = parts.next().context("no numeric part")?.parse()?;
                    let color = match parts.next().context("no color part")? {
                        "red" => &mut set.red,
                        "blue" => &mut set.blue,
                        "green" => &mut set.green,
                        other => return Err(anyhow::anyhow!("invalid color '{}'", other)),
                    };
                    *color += count;
                }
                Ok(set)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { id, sets })
    }
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    ["Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green
Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue
Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red
Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red
Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green"]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
