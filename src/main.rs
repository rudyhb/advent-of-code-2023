use utils::timer::Timer;

mod common;
mod day01_trebuchet;
mod day02_cube_conundrum;
mod day03_gear_ratios;
mod day04_scratchcards;
mod day05_if_you_give_a_seed_a_fertilizer;
mod day06_wait_for_it;
mod day07_camel_cards;
mod day08_haunted_wasteland;
mod day09_mirage_maintenance;
mod day10_pipe_maze;
mod day11_cosmic_expansion;
mod day12_hot_springs;
mod day13_point_of_incidence;
mod day14_parabolic_reflector_dish;
mod day15_lens_library;
mod day16_the_floor_will_be_lava;
mod day17_clumsy_crucible;
mod day18_lava_duct_lagoon;

fn main() {
    let mut context = common::Context::default();

    if let Ok(testing) = std::env::var("APP_TESTING") {
        if let Ok(testing) = testing.parse() {
            context.set_testing(testing);
        }
    }
    //context.set_testing(0);

    if std::env::var("RUST_LOG").is_err() {
        if context.is_testing() {
            std::env::set_var("RUST_LOG", "trace");
        } else {
            std::env::set_var("RUST_LOG", "info");
        }
    }
    env_logger::init();
    let _timer = Timer::start(|elapsed| println!("main took {} ms", elapsed.as_millis()));

    let days = days();

    let day: usize = if let Some(arg1) = std::env::args().nth(1) {
        arg1.parse().expect("Failed to parse day number")
    } else {
        days.len()
    };

    context.set_text_input(Box::new(move || {
        std::fs::read_to_string(format!("input/{:02}.txt", day))
            .unwrap_or_else(|_| panic!("Failed to read input file input/{:02}.txt", day))
            .into()
    }));
    let run = days[day - 1];

    println!("Running day {}\n", day);
    run(&mut context);
}

fn days() -> &'static [fn(&mut common::Context)] {
    &[
        day01_trebuchet::run,
        day02_cube_conundrum::run,
        day03_gear_ratios::run,
        day04_scratchcards::run,
        day05_if_you_give_a_seed_a_fertilizer::run,
        day06_wait_for_it::run,
        day07_camel_cards::run,
        day08_haunted_wasteland::run,
        day09_mirage_maintenance::run,
        day10_pipe_maze::run,
        day11_cosmic_expansion::run,
        day12_hot_springs::run,
        day13_point_of_incidence::run,
        day14_parabolic_reflector_dish::run,
        day15_lens_library::run,
        day16_the_floor_will_be_lava::run,
        day17_clumsy_crucible::run,
        day18_lava_duct_lagoon::run,
    ]
}
