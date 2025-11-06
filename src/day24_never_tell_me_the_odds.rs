use crate::common::day_setup::{AppContext, Day};
use anyhow::Context;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Div, Mul, RangeInclusive, Sub};
use std::str::FromStr;

pub fn day() -> Day {
    Day::custom(custom).with_test_inputs(&[r#"19, 13, 30 @ -2,  1, -2
18, 19, 22 @ -1, -1, -2
20, 25, 34 @ -2, -2, -4
12, 31, 28 @ -1, -2, -1
20, 19, 15 @  1, -5, -3"#])
}

fn custom(context: &AppContext) {
    let test_area = if context.is_testing() {
        7.0f64..=27.0
    } else {
        200000000000000.0f64..=400000000000000.0
    };

    let hailstones: Vec<HailstoneInitial> = context
        .get_input()
        .lines()
        .map(|s| s.parse())
        .collect::<Result<_, _>>()
        .unwrap();

    let mut count = 0;
    for (i, a) in hailstones.iter().enumerate() {
        for b in hailstones.iter().skip(i + 1) {
            let intersection = a.intersect_with_2d(b, &test_area);
            log::debug!("\nA: {a}\nB: {b}\nPaths intersect {intersection}");
            if let Intersection::Inside(_) = intersection {
                count += 1;
            }
        }
    }

    println!("part 1: intersections inside: {count}");

    // solve_2 uses 3 stones. We can make sure we get the right answer
    let mut solutions = vec![];
    for i in 0..3 {
        solutions.push(solve_2(&hailstones[i..]));
    }

    assert_eq!(solutions[0], solutions[1]);
    assert_eq!(solutions[1], solutions[2]);

    let stone = &solutions[0];
    let sum = stone.position.x + stone.position.y + stone.position.z;
    println!("part 2: sum pos coords: {sum}");
}

fn solve_2(hailstones: &[HailstoneInitial]) -> HailstoneInitial {
    // let's change coordinates to be relative to the first hailstone
    // p1, v1 is always 0, 0, 0
    let p2 = hailstones[1].position - hailstones[0].position;
    let v2 = hailstones[1].velocity - hailstones[0].velocity;
    let p3 = hailstones[2].position - hailstones[0].position;
    let v3 = hailstones[2].velocity - hailstones[0].velocity;

    // collision 1 happens at 0,0,0
    // collision 2 happens at c2 = p2 + v2 * t2
    // collision 3 happens at c3 = p3 + v3 * t2

    // since the stone has to be thrown in a straight line,
    // the vectors from origin to c2 and origin to c3 are collinear.
    // collinear => cross product of c2 x c3 = 0
    // (p2 + v2 * t2) x (p3 + v3 * t3) = 0
    // => p2 x p3 + t3 * (p2 x v3) + t2 * (v2 x p3) + t2*t3 * (v2 x v3) = 0

    // take dot products with v2, v3 to solve for t2, t3

    // (p2 x p3) * v3 + t2 * (v2 x p3) * v3 = 0
    let t2 = -p2.cross(&p3).dot(&v3) / v2.cross(&p3).dot(&v3);
    // (p2 x p3) * v2 + t3 * (p2 x v3) * v2 = 0
    let t3 = -p2.cross(&p3).dot(&v2) / p2.cross(&v3).dot(&v2);

    // collision points
    let hit_2 = hailstones[1].position_at_int(t2);
    let hit_3 = hailstones[2].position_at_int(t3);

    let velocity = (hit_3 - hit_2) / (t3 - t2);
    let stone_thrown = HailstoneInitial {
        velocity,
        position: hit_2,
    };

    HailstoneInitial {
        velocity,
        position: stone_thrown.position_at_int(-t2),
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
struct Point3D<T> {
    x: T,
    y: T,
    z: T,
}

enum Intersection {
    Inside(Point3D<f64>),
    Outside(Point3D<f64>),
    InThePast(&'static str),
    Parallel,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct HailstoneInitial {
    position: Point3D<i128>,
    velocity: Point3D<i128>,
}

impl HailstoneInitial {
    pub fn position_at(&self, t: f64) -> Point3D<f64> {
        self.position.as_f64() + (self.velocity.as_f64() * t)
    }
    pub fn position_at_int(&self, t: i128) -> Point3D<i128> {
        self.position + (self.velocity * t)
    }
    pub fn intersect_with_2d(&self, other: &Self, range: &RangeInclusive<f64>) -> Intersection {
        // x = x0,1 + t1*vx,1 = x0,2 + t2*vx,2
        // y = y0,1 + t1*vy,1 = y0,2 + t2*vy,2
        // --> t2 = (y0,1 - y0,2 + t1*vy,1) / vy,2
        // --> t1 = (vy,2 * (x0,2 - x0,1) + vx,2 * (y0,1 - y0,2)) / (vx,1*vy,2 - vy,1*vx,2)

        let den = (self.velocity.x * other.velocity.y) - (self.velocity.y * other.velocity.x);
        if den == 0 {
            return Intersection::Parallel;
        }

        let t1 = ((other.velocity.y * (other.position.x - self.position.x))
            + (other.velocity.x * (self.position.y - other.position.y))) as f64
            / den as f64;
        let t2 = if other.velocity.x == 0 {
            (self.position.y as f64 + (t1 * self.velocity.y as f64) - other.position.y as f64)
                / other.velocity.y as f64
        } else {
            (self.position.x as f64 + (t1 * self.velocity.x as f64) - other.position.x as f64)
                / other.velocity.x as f64
        };

        match (t1.is_sign_negative(), t2.is_sign_negative()) {
            (true, true) => return Intersection::InThePast("both"),
            (true, false) => return Intersection::InThePast("A"),
            (false, true) => return Intersection::InThePast("B"),
            (false, false) => {}
        }

        let intersection = self.position_at(t1);

        if range.contains(&intersection.x) && range.contains(&intersection.y) {
            Intersection::Inside(intersection)
        } else {
            Intersection::Outside(intersection)
        }
    }
}

impl FromStr for HailstoneInitial {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('@');
        fn parse_part(s: &str) -> Result<Point3D<i128>, <HailstoneInitial as FromStr>::Err> {
            let mut parts = s.split(',');
            Ok(Point3D {
                x: parts.next().context("no x")?.trim().parse()?,
                y: parts.next().context("no y")?.trim().parse()?,
                z: parts.next().context("no z")?.trim().parse()?,
            })
        }
        Ok(Self {
            position: parse_part(parts.next().context("no position")?)?,
            velocity: parse_part(parts.next().context("no velocity")?)?,
        })
    }
}

impl Point3D<i128> {
    pub fn as_f64(self) -> Point3D<f64> {
        Point3D {
            x: self.x as f64,
            y: self.y as f64,
            z: self.z as f64,
        }
    }
}

impl<T> Point3D<T>
where
    T: Add<T, Output = T> + Sub<T, Output = T> + Mul<T, Output = T> + Copy,
{
    pub fn cross(&self, other: &Self) -> Self {
        Point3D {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
    pub fn dot(&self, other: &Self) -> T {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

impl<T> Add<Point3D<T>> for Point3D<T>
where
    T: Add<T, Output = T>,
{
    type Output = Point3D<T>;

    fn add(self, rhs: Point3D<T>) -> Self::Output {
        Point3D {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<T> Sub<Point3D<T>> for Point3D<T>
where
    T: Sub<T, Output = T>,
{
    type Output = Point3D<T>;

    fn sub(self, rhs: Point3D<T>) -> Self::Output {
        Point3D {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl<T, V> Mul<V> for Point3D<T>
where
    T: Mul<V, Output = T>,
    V: Copy,
{
    type Output = Self;

    fn mul(self, rhs: V) -> Self::Output {
        Point3D {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl<T, V> Div<V> for Point3D<T>
where
    T: Div<V, Output = T>,
    V: Copy,
{
    type Output = Self;

    fn div(self, rhs: V) -> Self::Output {
        Point3D {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl<T: Display> Display for Point3D<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}, {}", self.x, self.y, self.z)
    }
}

impl Display for Intersection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Intersection::Inside(val) => write!(f, "*inside* (at {})", val),
            Intersection::Outside(val) => write!(f, "outside (at {})", val),
            Intersection::InThePast(val) => write!(f, "in the past for {}", val),
            Intersection::Parallel => write!(f, "never - parallel"),
        }
    }
}

impl Display for HailstoneInitial {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} @ {}", self.position, self.velocity)
    }
}
