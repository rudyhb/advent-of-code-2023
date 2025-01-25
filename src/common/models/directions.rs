use crate::common::models::Point;
use bitflags::bitflags;
use std::fmt::{Display, Formatter};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct DirectionFlag: u8 {
        const UP_LEFT = 0b00000001;
        const UP = 0b00000010;
        const UP_RIGHT = 0b00000100;
        const RIGHT = 0b00001000;
        const DOWN_RIGHT = 0b00010000;
        const DOWN = 0b00100000;
        const DOWN_LEFT = 0b01000000;
        const LEFT = 0b10000000;

        const FOUR_DIRECTIONS = Self::UP.bits() | Self::DOWN.bits() | Self::LEFT.bits() | Self::RIGHT.bits();
        const ALL_DIRECTIONS = 0b11111111;
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn from_vec(from: &Point<usize>, to: &Point<usize>) -> Option<Self> {
        let dx = (to.x as isize) - (from.x as isize);
        let dy = (to.y as isize) - (from.y as isize);
        if (dx == 0) == (dy == 0) {
            return None;
        }
        Some(match dx.signum() {
            -1 => Direction::Left,
            1 => Direction::Right,
            0 => match dy.signum() {
                -1 => Direction::Up,
                1 => Direction::Down,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        })
    }
    pub fn invert(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
    pub fn turn_right(&self) -> Self {
        match self {
            Direction::Up => Direction::Right,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
            Direction::Right => Direction::Down,
        }
    }
    pub fn turn_left(&self) -> Self {
        match self {
            Direction::Up => Direction::Left,
            Direction::Down => Direction::Right,
            Direction::Left => Direction::Down,
            Direction::Right => Direction::Up,
        }
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Up => write!(f, "^"),
            Direction::Down => write!(f, "v"),
            Direction::Left => write!(f, "<"),
            Direction::Right => write!(f, ">"),
        }
    }
}
