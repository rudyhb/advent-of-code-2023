use crate::common::day_setup::Day;
use common::day_setup;
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
mod day19_aplenty;
mod day20_pulse_propagation;
mod day21_step_counter;
mod day22_sand_slabs;
mod day23_a_long_walk;
mod day24_never_tell_me_the_odds;

fn main() {
    let mut context = day_setup::AppContext::default();

    context.set_testing(
        std::env::var("APP_TESTING")
            .ok()
            .map(|s| s.parse().expect("invalid APP_TESTING")),
    );
    //context.set_testing(Some(0));

    if std::env::var("RUST_LOG").is_err() {
        unsafe {
            if context.is_testing() {
                std::env::set_var("RUST_LOG", "debug");
            } else {
                std::env::set_var("RUST_LOG", "info");
            }
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
    let run = days[day - 1]();

    println!("Running day {}\n", day);
    run.exec(&mut context);
}

fn days() -> &'static [fn() -> Day] {
    &[
        day01_trebuchet::day,
        day02_cube_conundrum::day,
        day03_gear_ratios::day,
        day04_scratchcards::day,
        day05_if_you_give_a_seed_a_fertilizer::day,
        day06_wait_for_it::day,
        day07_camel_cards::day,
        day08_haunted_wasteland::day,
        day09_mirage_maintenance::day,
        day10_pipe_maze::day,
        day11_cosmic_expansion::day,
        day12_hot_springs::day,
        day13_point_of_incidence::day,
        day14_parabolic_reflector_dish::day,
        day15_lens_library::day,
        day16_the_floor_will_be_lava::day,
        day17_clumsy_crucible::day,
        day18_lava_duct_lagoon::day,
        day19_aplenty::day,
        day20_pulse_propagation::day,
        day21_step_counter::day,
        day22_sand_slabs::day,
        day23_a_long_walk::day,
        day24_never_tell_me_the_odds::day,
    ]
}
