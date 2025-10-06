use crate::common::day_setup::Day;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{Index, IndexMut};
use std::str::FromStr;
pub fn day() -> Day {
    Day::new(run).with_test_inputs(&["1,0,1~1,2,1
0,0,2~2,0,2
0,2,3~2,2,3
0,0,4~0,2,4
2,0,5~2,2,5
0,1,6~2,1,6
1,1,8~1,1,9"])
}

pub fn run(input: &str) {
    let bricks: Vec<Brick> = input.lines().map(|line| line.parse().unwrap()).collect();
    let tower = BrickTower::build_from(bricks);
    println!(
        "part 1: bricks safely disintegrated = {}",
        tower.safely_disintegrated().count()
    );
    println!(
        "part 2: sum number of other bricks that would fall = {}",
        tower.sum_fallen_bricks_after_disintegration()
    );
}

#[derive(Default)]
struct BrickTower {
    bricks: Vec<BrickStatus>,
    xy_z: HashMap<(u32, u32), BTreeMap<u32, usize>>,
}

impl BrickTower {
    pub fn build_from(bricks: Vec<Brick>) -> Self {
        let mut tower = Self::default();
        let mut letter = b'A' - 1;
        let mut bricks: Vec<_> = bricks
            .into_iter()
            .map(|brick| {
                letter += 1;
                (brick, letter as char)
            })
            .collect();
        bricks.sort_by(|(a, _), (b, _)| a.position.z.cmp(&b.position.z));

        for (brick, name) in bricks {
            tower.settle_brick(brick, name);
        }

        for brick in tower.bricks.iter() {
            let support: Vec<_> = brick
                .support
                .iter()
                .map(|&i| tower.bricks[i].name)
                .collect();
            let supported_by: Vec<_> = brick
                .supported_by
                .iter()
                .map(|&i| tower.bricks[i].name)
                .collect();
            log::debug!(
                "Brick {} supports {:?} and is supported by {:?}",
                brick.name,
                support,
                supported_by
            );
        }

        tower
    }
    fn settle_brick(&mut self, mut brick: Brick, name: char) {
        let mut max_supporters: Vec<_> = brick
            .get_lowest_points()
            .into_iter()
            .filter_map(|point| {
                self.xy_z
                    .get(&(point.x, point.y))
                    .and_then(|z_points| z_points.range(0..point.z).next_back())
            })
            .collect();
        let max = max_supporters
            .iter()
            .map(|(z, _)| **z)
            .max()
            .unwrap_or_default(); // default 0 is floor
        max_supporters.retain(|val| *val.0 == max);
        let translate_size = brick.position.z - max - 1;

        brick.position = brick.position.translate(
            translate_size,
            Direction3D {
                axis: Axis::Z,
                negative: true,
            },
        );

        let status = BrickStatus {
            brick,
            supported_by: max_supporters.iter().map(|(_, index)| **index).collect(),
            support: Default::default(),
            name,
        };

        let index = self.bricks.len();

        for &supporter in status.supported_by.iter() {
            self.bricks[supporter].support.insert(index);
        }

        for top_point in status.brick.get_highest_points() {
            self.xy_z
                .entry((top_point.x, top_point.y))
                .or_default()
                .insert(top_point.z, index);
        }

        self.bricks.push(status);
    }
    pub fn safely_disintegrated(&self) -> impl Iterator<Item = &Brick> {
        self.bricks
            .iter()
            .enumerate()
            .filter(|(i, brick)| self.brick_safely_disintegrated(brick, *i))
            .map(|(_, brick)| &brick.brick)
    }
    fn brick_safely_disintegrated(&self, brick: &BrickStatus, index: usize) -> bool {
        brick.support.iter().all(|&supported| {
            assert!(self.bricks[supported].supported_by.contains(&index));
            self.bricks[supported].supported_by.len() > 1
        })
    }
    pub fn sum_fallen_bricks_after_disintegration(&self) -> usize {
        (0..self.bricks.len())
            .map(|i| self.fallen_bricks_after_disintegrating(i))
            .sum()
    }
    fn fallen_bricks_after_disintegrating(&self, brick: usize) -> usize {
        let mut fallen = HashSet::from([brick]);
        let get_z = |i: usize| self.bricks[i].brick.position.z;
        let push = |set: &mut HashSet<usize>, next: usize| {
            set.insert(next);
        };
        let pop_lowest_z = |set: &mut HashSet<usize>| -> Option<usize> {
            let mut vec: Vec<_> = set.iter().copied().collect();
            vec.sort_unstable_by(|&a, &b| get_z(b).cmp(&get_z(a)));
            vec.pop().inspect(|val| {
                set.remove(val);
            })
        };

        let mut children = HashSet::from_iter(self.bricks[brick].support.iter().copied());

        while let Some(next) = pop_lowest_z(&mut children) {
            if self.bricks[next]
                .supported_by
                .iter()
                .all(|supporter| fallen.contains(supporter))
            {
                fallen.insert(next);
                self.bricks[next]
                    .support
                    .iter()
                    .for_each(|&i| push(&mut children, i));
            }
        }

        let fallen = fallen.len() - 1; // remove self
        if fallen > 0 {
            log::debug!(
                "disintegrating brick {} would cause {} other bricks to fall",
                self.bricks[brick].name,
                fallen
            );
        }
        fallen
    }
}

struct BrickStatus {
    brick: Brick,
    supported_by: HashSet<usize>,
    support: HashSet<usize>,
    name: char,
}

struct Brick {
    position: Coord,
    size: u32,
    extend_direction: Direction3D,
}

impl Brick {
    pub fn new(position: Coord, size: u32, extend_direction: Direction3D) -> Self {
        if extend_direction.negative && extend_direction.axis == Axis::Z {
            panic!("invalid state - brick z should be lowest point");
        }
        Self {
            position,
            size,
            extend_direction,
        }
    }
    pub fn get_highest_points(&self) -> Vec<Coord> {
        if self.extend_direction.axis == Axis::Z {
            if self.extend_direction.negative {
                unreachable!()
                //vec![self.position.clone()]
            } else {
                vec![
                    self.position
                        .clone()
                        .translate(self.size - 1, self.extend_direction),
                ]
            }
        } else {
            self.get_all_points()
        }
    }
    pub fn get_lowest_points(&self) -> Vec<Coord> {
        if self.extend_direction.axis == Axis::Z {
            if self.extend_direction.negative {
                unreachable!()
                //vec![self.position.clone().translate(self.size, self.extend_direction)]
            } else {
                vec![self.position.clone()]
            }
        } else {
            self.get_all_points()
        }
    }
    fn get_all_points(&self) -> Vec<Coord> {
        let mut result = Vec::with_capacity(self.size as usize);
        result.push(self.position.clone());
        for _ in 1..self.size {
            result.push(
                result
                    .last()
                    .unwrap()
                    .clone()
                    .translate(1, self.extend_direction),
            );
        }
        assert_eq!(self.size as usize, result.len());
        result
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct Coord {
    x: u32,
    y: u32,
    z: u32,
}

impl Coord {
    pub fn translate(mut self, size: u32, extend_direction: Direction3D) -> Self {
        if extend_direction.negative {
            self[extend_direction.axis] -= size;
        } else {
            self[extend_direction.axis] += size;
        }
        self
    }
}

#[derive(Copy, Debug, Clone, Eq, PartialEq, Hash)]
struct Direction3D {
    axis: Axis,
    negative: bool,
}

#[derive(Default, Copy, Debug, Clone, Eq, PartialEq, Hash)]
enum Axis {
    #[default]
    X,
    Y,
    Z,
}

impl Index<Axis> for Coord {
    type Output = u32;

    fn index(&self, index: Axis) -> &Self::Output {
        match index {
            Axis::X => &self.x,
            Axis::Y => &self.y,
            Axis::Z => &self.z,
        }
    }
}

impl IndexMut<Axis> for Coord {
    fn index_mut(&mut self, index: Axis) -> &mut Self::Output {
        match index {
            Axis::X => &mut self.x,
            Axis::Y => &mut self.y,
            Axis::Z => &mut self.z,
        }
    }
}

impl FromStr for Brick {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let [left, right] = s.trim().split("~").collect::<Vec<_>>()[..] else {
            return Err(anyhow::anyhow!("not a brick format: '{}'", s));
        };
        let mut left: Coord = left.parse()?;
        let mut right: Coord = right.parse()?;
        if right.z < left.z {
            std::mem::swap(&mut left, &mut right);
        }

        let mut axes = [Axis::X, Axis::Y, Axis::Z]
            .iter()
            .filter(|&&axis| left[axis] != right[axis]);
        let axis = axes.next().copied().unwrap_or_default();
        if axes.next().is_some() {
            return Err(anyhow::anyhow!(
                "block extends in multiple directions... : {}",
                s
            ));
        }

        let size = left[axis].abs_diff(right[axis]) + 1;
        let negative = left[axis] > right[axis];

        Ok(Self::new(left, size, Direction3D { axis, negative }))
    }
}

impl FromStr for Coord {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let [x, y, z] = s.trim().split(",").collect::<Vec<_>>()[..] else {
            return Err(anyhow::anyhow!("not a coord format: '{}'", s));
        };
        Ok(Self {
            x: x.parse()?,
            y: y.parse()?,
            z: z.parse()?,
        })
    }
}
