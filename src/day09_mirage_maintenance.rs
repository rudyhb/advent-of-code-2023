use crate::common::day_setup::Day;
use std::str::FromStr;

pub fn day() -> Day {
    Day::new(run).with_test_inputs(&["0 3 6 9 12 15
1 3 6 10 15 21
10 13 16 21 30 45"])
}
pub fn run(input: &str) {
    let histories: Vec<History> = input.lines().map(|line| line.parse().unwrap()).collect();

    let sum: i64 = histories
        .iter()
        .map(|history| {
            let next = history.next_value();
            log::debug!("{:?} --> {}", &history.0, next);
            next
        })
        .sum();
    println!("sum part 1: {}", sum);

    let sum: i64 = histories
        .iter()
        .map(|history| {
            let previous = history.previous_value();
            log::debug!("{} <-- {:?}", previous, &history.0);
            previous
        })
        .sum();
    println!("sum part 2: {}", sum);
}

struct History(Vec<i64>);

impl History {
    pub fn next_value(&self) -> i64 {
        let mut last_values = Vec::new();
        let mut values = self.0.clone();
        while values.iter().any(|&v| v != 0) {
            assert!(values.len() > 1, "no values left for diff");
            last_values.push(values.last().copied().unwrap());
            values = values
                .windows(2)
                .map(|values| values[1] - values[0])
                .collect();
        }

        last_values.iter().sum()
    }
    pub fn previous_value(&self) -> i64 {
        let mut first_values = Vec::new();
        let mut values = self.0.clone();
        while values.iter().any(|&v| v != 0) {
            assert!(values.len() > 1, "no values left for diff");
            first_values.push(values.first().copied().unwrap());
            values = values
                .windows(2)
                .map(|values| values[1] - values[0])
                .collect();
        }

        first_values.iter().rev().fold(0, |sum, &next| {
            log::debug!("{} - {}", next, sum);
            next - sum
        })
    }
}

impl FromStr for History {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            s.split_whitespace()
                .map(|val| val.parse())
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }
}
