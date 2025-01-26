use crate::common::models::Direction;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use utils::a_star::Node;
use utils::common::NumericWithUnitValue;

#[derive(Clone)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl Point<usize> {
    #[allow(dead_code)]
    #[inline]
    pub fn move_in_unchecked(&self, direction: Direction) -> Self {
        self.move_in_times(direction, 1).unwrap()
    }
    pub fn move_in_times(&self, direction: Direction, times: usize) -> Option<Self> {
        match direction {
            Direction::Up => {
                if self.y >= times {
                    Some(Self {
                        x: self.x,
                        y: self.y - times,
                    })
                } else {
                    None
                }
            }
            Direction::Down => Some(Self {
                x: self.x,
                y: self.y + times,
            }),
            Direction::Left => {
                if self.x >= times {
                    Some(Self {
                        x: self.x - times,
                        y: self.y,
                    })
                } else {
                    None
                }
            }
            Direction::Right => Some(Self {
                x: self.x + times,
                y: self.y,
            }),
        }
    }
    #[inline]
    pub fn move_in(&self, direction: Direction) -> Option<Self> {
        self.move_in_times(direction, 1)
    }
    pub fn manhattan_distance(&self, other: &Self) -> usize {
        (self.x.max(other.x) - self.x.min(other.x)) + (self.y.max(other.y) - self.y.min(other.y))
    }
}

impl<T: Display> Display for Point<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

impl<T: Debug> Debug for Point<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{x: {:?}, y: {:?}}}", self.x, self.y)
    }
}

impl<T: Hash> Hash for Point<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
    }
}

impl<T: PartialEq> PartialEq for Point<T> {
    fn eq(&self, other: &Self) -> bool {
        self.y.eq(&other.y) && self.x.eq(&other.x)
    }
}

impl<T: Eq> Eq for Point<T> {}

impl<T: PartialOrd> PartialOrd for Point<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let cmp = self.y.partial_cmp(&other.y);
        if let Some(Ordering::Equal) = cmp {
            self.x.partial_cmp(&other.x)
        } else {
            cmp
        }
    }
}

impl<T: Ord> Ord for Point<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        let cmp = self.y.cmp(&other.y);
        if let Ordering::Equal = cmp {
            self.x.cmp(&other.x)
        } else {
            cmp
        }
    }
}

impl Node for Point<usize> {}

impl<T: NumericWithUnitValue> Point<T> {
    pub fn move_in_direction_unchecked(&self, direction: Direction) -> Self {
        match direction {
            Direction::Up => Self {
                x: self.x,
                y: self.y - T::unit(),
            },
            Direction::Down => Self {
                x: self.x,
                y: self.y + T::unit(),
            },
            Direction::Left => Self {
                x: self.x - T::unit(),
                y: self.y,
            },
            Direction::Right => Self {
                x: self.x + T::unit(),
                y: self.y,
            },
        }
    }
}
