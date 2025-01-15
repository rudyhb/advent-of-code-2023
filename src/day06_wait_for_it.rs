use crate::common::{Context, InputProvider};

pub fn run(context: &mut Context) {
    context.add_test_inputs(get_test_inputs());

    let input = context.get_input();

    let races = parse(&input);

    let product: u64 = races.iter().map(ways_to_beat_record).product();
    println!("product part 1: {}", product);

    let race = parse2(&input);
    let ways = ways_to_beat_record(&race);
    println!("ways to beat big race: {}", ways);
}

fn ways_to_beat_record(race: &Race) -> u64 {
    log::debug!("race: {:?}", race);

    // to tie: Tx - x^2 - d = 0
    let bounds_exclusive = solve_quadratic(1.0, -(race.time as f64), race.best_distance as f64);
    log::debug!("solution to quadratic {:?}", bounds_exclusive);
    let bounds_exclusive = [
        bounds_exclusive[0].floor() as u64,
        bounds_exclusive[1].ceil() as u64,
    ];
    log::debug!(
        "can match record with ({}) {:?}",
        bounds_exclusive[1] - bounds_exclusive[0] - 1,
        bounds_exclusive
    );
    bounds_exclusive[1] - bounds_exclusive[0] - 1
}

fn solve_quadratic(a: f64, b: f64, c: f64) -> [f64; 2] {
    let rhs = (b.powi(2) - 4.0 * a * c).sqrt();
    [(-b - rhs) / (2.0 * a), (-b + rhs) / (2.0 * a)]
}

#[derive(Debug)]
struct Race {
    time: u64,
    best_distance: u64,
}

fn parse(input: &str) -> Vec<Race> {
    let mut lines = input.lines();
    let parts1 = lines.next().unwrap().split_whitespace();
    let parts2 = lines.next().unwrap().split_whitespace();
    parts1
        .zip(parts2)
        .skip(1)
        .map(|(time, distance)| Race {
            time: time.parse().unwrap(),
            best_distance: distance.parse().unwrap(),
        })
        .collect()
}

fn parse2(input: &str) -> Race {
    let mut lines = input.lines();
    let mut combine_numbers = || -> u64 {
        let numbers = lines.next().unwrap().split(":").nth(1).unwrap();
        let numbers = numbers.replace(|c: char| c.is_whitespace(), "");
        numbers.parse().unwrap()
    };
    Race {
        time: combine_numbers(),
        best_distance: combine_numbers(),
    }
}

fn get_test_inputs() -> impl Iterator<Item = Box<InputProvider>> {
    ["Time:      7  15   30
Distance:  9  40  200"]
    .into_iter()
    .map(|input| Box::new(move || input.into()) as Box<InputProvider>)
}
