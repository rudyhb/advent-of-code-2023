use crate::common::models::Direction;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use utils::a_star::Node;

#[derive(Clone)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl Point<usize> {
    #[allow(dead_code)]
    pub fn move_in_unchecked(&self, direction: Direction) -> Self {
        self.move_in(direction).unwrap()
    }
    pub fn move_in(&self, direction: Direction) -> Option<Self> {
        match direction {
            Direction::Up => {
                if self.y > 0 {
                    Some(Self {
                        x: self.x,
                        y: self.y - 1,
                    })
                } else {
                    None
                }
            }
            Direction::Down => Some(Self {
                x: self.x,
                y: self.y + 1,
            }),
            Direction::Left => {
                if self.x > 0 {
                    Some(Self {
                        x: self.x - 1,
                        y: self.y,
                    })
                } else {
                    None
                }
            }
            Direction::Right => Some(Self {
                x: self.x + 1,
                y: self.y,
            }),
        }
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
