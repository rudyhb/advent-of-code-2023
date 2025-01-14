use crate::common::{Context, InputProvider};
use anyhow::Context as AnyhowContext;
use std::collections::HashSet;
use std::str::FromStr;

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let input = context.get_input();
    let input = input.as_str();

    let cards: Vec<Card> = input
        .lines()
        .map(|line| line.parse())
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let sum: u32 = cards.iter().map(|card| card.get_points()).sum();
    println!("part 1 sum: {}", sum);

    let mut cards: Vec<_> = cards.into_iter().map(|card| (1usize, card)).collect();
    for i in 0..cards.len() {
        let (copies, card) = &cards[i];
        let copies = *copies;
        let count = card.get_winning_count();
        for (card_count, _) in cards[i + 1..i + 1 + count].iter_mut() {
            *card_count += copies;
        }
    }

    let count: usize = cards.iter().map(|(count, _)| *count).sum();
    println!("part 2 total scratchcards: {}", count);
}

struct Card {
    winning: HashSet<u32>,
    playing: Vec<u32>,
}

impl Card {
    pub fn get_points(&self) -> u32 {
        let scoring = self.get_winning_count() as u32;
        if scoring > 0 {
            2u32.pow(scoring - 1)
        } else {
            0
        }
    }
    pub fn get_winning_count(&self) -> usize {
        self.playing
            .iter()
            .filter(|playing| self.winning.contains(playing))
            .count()
    }
}

impl FromStr for Card {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(":");
        let _id: u32 = parts
            .next()
            .unwrap()
            .split_whitespace()
            .last()
            .context("no card id")?
            .parse()?;

        let mut parts = parts.next().context("no card body")?.split("|");

        let winning = parts
            .next()
            .unwrap()
            .split_whitespace()
            .map(|word| word.parse())
            .collect::<Result<HashSet<u32>, _>>()?;

        let playing = parts
            .next()
            .context("no playing numbers")?
            .split_whitespace()
            .map(|word| word.parse())
            .collect::<Result<Vec<u32>, _>>()?;

        Ok(Self { winning, playing })
    }
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    ["Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53
Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19
Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1
Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83
Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36
Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11"]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
