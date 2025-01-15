use crate::common::{Context, InputProvider};
use anyhow::Context as AnyhowContext;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let input = context.get_input();

    let mut hands: Vec<Hand> = input
        .lines()
        .map(|line| line.parse())
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    hands.sort_unstable();

    let total_winnings = calculate_winnings(&hands);
    println!("part 1 winnings: {}", total_winnings);

    for hand in hands.iter_mut() {
        hand.convert_j_to_joker();
    }
    hands.sort_unstable();

    let total_winnings = calculate_winnings(&hands);
    println!("part 2 winnings: {}", total_winnings);
}

fn calculate_winnings(sorted_hands: &[Hand]) -> u64 {
    sorted_hands
        .iter()
        .enumerate()
        .map(|(i, hand)| {
            let rank = (i + 1) as u64;
            log::debug!("hand {} gets rank {}", hand, rank);
            rank * hand.bid
        })
        .sum()
}

#[derive(Clone, Eq, PartialEq)]
struct Hand {
    cards: [Card; 5],
    hand_type: HandType,
    bid: u64,
}

impl Hand {
    fn new(cards: [Card; 5], bid: u64) -> Self {
        let hand_type = Self::calculate_hand_type(&cards);
        Self {
            cards,
            hand_type,
            bid,
        }
    }
    fn calculate_hand_type(cards: &[Card]) -> HandType {
        let ranks: HashMap<Card, usize> = cards.iter().filter(|&&card| card != Card::Joker).fold(
            HashMap::default(),
            |mut acc, &next| {
                *acc.entry(next).or_default() += 1;
                acc
            },
        );
        let mut ranks: Vec<_> = ranks.into_values().collect();
        ranks.sort_unstable();

        let joker_count = cards.iter().filter(|&&card| card == Card::Joker).count();
        if let Some(last) = ranks.last_mut() {
            *last += joker_count;
        } else {
            ranks = vec![joker_count];
        }

        if ranks.last() == Some(&5) {
            return HandType::FiveOfAKind;
        }

        let last = ranks[ranks.len() - 1];
        let second_last = ranks[ranks.len() - 2];

        match last {
            1 => HandType::HighCard,
            2 => {
                if second_last == 2 {
                    HandType::TwoPair
                } else {
                    HandType::OnePair
                }
            }
            3 => {
                if second_last == 2 {
                    HandType::FullHouse
                } else {
                    HandType::ThreeOfAKind
                }
            }
            4 => HandType::FourOfAKind,
            _ => unreachable!(),
        }
    }
    pub fn convert_j_to_joker(&mut self) {
        for card in self.cards.iter_mut() {
            if *card == Card::J {
                *card = Card::Joker;
            }
        }
        self.hand_type = Self::calculate_hand_type(&self.cards);
    }
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        let cmp = self.hand_type.cmp(&other.hand_type);
        if cmp != Ordering::Equal {
            return cmp;
        }
        for (left, right) in self.cards.iter().zip(other.cards.iter()) {
            let cmp = left.cmp(right);
            if cmp != Ordering::Equal {
                return cmp;
            }
        }

        Ordering::Equal
    }
}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl FromStr for Hand {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();
        Ok(Self::new(
            parts
                .next()
                .context("empty input")?
                .chars()
                .map(|c| c.into())
                .collect::<Vec<Card>>()
                .try_into()
                .ok()
                .context("hand should be length 5")?,
            parts.next().context("no bid found")?.parse()?,
        ))
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
enum HandType {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
enum Card {
    A,
    K,
    Q,
    J,
    Num(u8),
    Joker,
}

impl Card {
    const fn rank(&self) -> u8 {
        match self {
            Card::A => 14,
            Card::K => 13,
            Card::Q => 12,
            Card::J => 11,
            Card::Num(val) => *val,
            Card::Joker => 1,
        }
    }
}

impl Ord for Card {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank().cmp(&other.rank())
    }
}

impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<char> for Card {
    fn from(value: char) -> Self {
        match value {
            'A' => Self::A,
            'K' => Self::K,
            'Q' => Self::Q,
            'J' => Self::J,
            'T' => Self::Num(10),
            '2'..='9' => Self::Num(value.to_digit(10).unwrap() as u8),
            other => panic!("invalid card rank '{}'", other),
        }
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Card::A => {
                write!(f, "A")
            }
            Card::K => {
                write!(f, "K")
            }
            Card::Q => {
                write!(f, "Q")
            }
            Card::J => {
                write!(f, "J")
            }
            Card::Joker => {
                write!(f, "*")
            }
            Card::Num(val) => {
                if *val == 10 {
                    write!(f, "T")
                } else {
                    write!(f, "{}", val)
                }
            }
        }
    }
}

impl Display for Hand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for card in self.cards.iter() {
            write!(f, "{}", card)?;
        }
        Ok(())
    }
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    ["32T3K 765
T55J5 684
KK677 28
KTJJT 220
QQQJA 483"]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
