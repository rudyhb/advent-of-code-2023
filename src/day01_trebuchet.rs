use crate::common::{Context, InputProvider};
use std::collections::HashMap;

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let input = context.get_input();
    let input = input.as_str();

    let mut sum = 0;
    for word in input.lines() {
        let first = if let Some(first) = word.chars().find(|c| c.is_numeric()) {
            first
        } else {
            continue;
        };
        let last = word.chars().filter(|c| c.is_numeric()).last().unwrap();
        let value = format!("{}{}", first, last).parse::<u32>().unwrap();
        sum += value;
    }

    println!("part 1 sum: {}", sum);

    let map = HashMap::from([
        ("one", "1"),
        ("two", "2"),
        ("three", "3"),
        ("four", "4"),
        ("five", "5"),
        ("six", "6"),
        ("seven", "7"),
        ("eight", "8"),
        ("nine", "9"),
    ]);

    let mut sum = 0;
    for word in input.lines() {
        let mut first = None;
        let mut last = None;

        for i in 0..word.len() {
            let word = &word[i..];
            let value = if let Some(value) = word
                .chars()
                .map(|c| if c.is_numeric() { Some(c) } else { None })
                .next()
                .flatten()
            {
                value.to_string()
            } else if let Some(value) = map
                .iter()
                .filter(|(&text, _)| word.starts_with(text))
                .map(|(_, value)| value)
                .next()
            {
                value.to_string()
            } else {
                continue;
            };

            if first.is_none() {
                first = Some(value.clone());
            }
            last = Some(value);
        }

        let value = format!("{}{}", first.unwrap(), last.unwrap())
            .parse::<u32>()
            .unwrap();
        log::debug!("value: {}", value);
        sum += value;
    }

    println!("part 2 sum: {}", sum);
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    [
        "1abc2
pqr3stu8vwx
a1b2c3d4e5f
treb7uchet",
        "two1nine
eightwothree
abcone2threexyz
xtwone3four
4nineeightseven2
zoneight234
7pqrstsixteen",
        "nineight",
    ]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
