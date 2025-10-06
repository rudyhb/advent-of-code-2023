use crate::common::day_setup::Day;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

pub fn day() -> Day {
    Day::new(run).with_test_inputs(&["rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7"])
}
pub fn run(input: &str) {
    let sum: u64 = input
        .split(",")
        .map(|line| <&str as Into<AsciiString>>::into(line).get_hash() as u64)
        .sum();
    println!("part 1 sum: {}", sum);

    solve(input);
}

fn solve(input: &str) {
    let mut boxes = LensBoxes::default();
    for lens in input.split(",").map(|s| s.parse::<LensInput>().unwrap()) {
        log::debug!("after \"{}\"", lens);
        boxes.next(lens);
        log::debug!("\n{}", boxes);
        log::trace!("\n{:?}", boxes);
    }
    println!("part 2 focusing power sum: {}", boxes.get_focusing_power());
}

struct LensBoxes {
    boxes: [LensBox; 256],
}

impl Default for LensBoxes {
    fn default() -> Self {
        Self {
            boxes: core::array::from_fn(|_| Default::default()),
        }
    }
}

impl LensBoxes {
    pub fn next(&mut self, lens: LensInput) {
        let box_id = lens.label.get_hash();
        let box_ = &mut self.boxes[box_id as usize];
        if let Operation::Remove = &lens.operation {
            box_.remove(&lens.label);
        } else {
            box_.add(lens.try_into().unwrap());
        }
    }
    pub fn get_focusing_power(&self) -> u64 {
        self.boxes
            .iter()
            .enumerate()
            .map(|(i, box_)| box_.get_focusing_power(i))
            .sum()
    }
}

#[derive(Default, Debug)]
struct LensBox {
    lenses: Vec<Lens>,
    indices: HashMap<AsciiString, usize>,
}

impl LensBox {
    pub fn remove(&mut self, label: &AsciiString) -> Option<Lens> {
        if let Some(&index) = self.indices.get(label) {
            let lens = self.lenses.remove(index);
            for other in &self.lenses[index..] {
                *self.indices.get_mut(&other.label).unwrap() -= 1;
            }
            self.indices.remove(label);
            Some(lens)
        } else {
            None
        }
    }
    pub fn add(&mut self, mut lens: Lens) -> Option<Lens> {
        if let Some(&index) = self.indices.get(&lens.label) {
            std::mem::swap(&mut lens, &mut self.lenses[index]);
            Some(lens)
        } else {
            let index = self.lenses.len();
            self.indices.insert(lens.label.clone(), index);
            self.lenses.push(lens);

            None
        }
    }
    pub fn get_focusing_power(&self, box_number: usize) -> u64 {
        self.lenses
            .iter()
            .enumerate()
            .map(|(i, lens)| lens.get_focusing_power(box_number, i))
            .sum()
    }
}

struct Lens {
    label: AsciiString,
    focal_length: u8,
}

impl Lens {
    pub fn get_focusing_power(&self, box_number: usize, lens_number: usize) -> u64 {
        (1 + box_number as u64) * (1 + lens_number as u64) * self.focal_length as u64
    }
}

struct LensInput {
    label: AsciiString,
    operation: Operation,
}

impl TryFrom<LensInput> for Lens {
    type Error = anyhow::Error;

    fn try_from(value: LensInput) -> Result<Self, Self::Error> {
        match value.operation {
            Operation::Remove => Err(anyhow::anyhow!("cannot convert {} to a lens", value)),
            Operation::Add(focal_length) => Ok(Self {
                label: value.label,
                focal_length,
            }),
        }
    }
}

enum Operation {
    Remove,
    Add(u8),
}

#[derive(Default, Eq, PartialEq, Hash, Clone)]
struct AsciiString(Vec<char>);

impl AsciiString {
    pub fn get_hash(&self) -> u8 {
        let mut value = 0u8;
        for &c in self.0.iter() {
            value = value.wrapping_add(c as u8);
            value = value.wrapping_mul(17);
        }
        value
    }
}

impl<'a> From<&'a str> for AsciiString {
    fn from(value: &'a str) -> Self {
        Self(value.chars().collect())
    }
}

impl FromStr for LensInput {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.ends_with('-') {
            return Ok(Self {
                label: AsciiString(s.chars().take_while(|c| c.is_alphabetic()).collect()),
                operation: Operation::Remove,
            });
        }
        let [label, op_val] = s.split("=").collect::<Vec<_>>()[..] else {
            return Err(anyhow::anyhow!("invalid lens string '{}'", s));
        };
        let op_val: u8 = op_val.parse()?;
        if !(1..=9).contains(&op_val) {
            return Err(anyhow::anyhow!("invalid op val '{}'", op_val));
        }
        Ok(Self {
            label: label.into(),
            operation: Operation::Add(op_val),
        })
    }
}

impl Display for LensInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label.0.iter().collect::<String>())?;
        match &self.operation {
            Operation::Remove => {
                write!(f, "-")
            }
            Operation::Add(val) => {
                write!(f, "={}", val)
            }
        }
    }
}

impl Display for LensBox {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for lens in &self.lenses {
            write!(f, "  {}", lens)?;
        }
        Ok(())
    }
}

impl Display for Lens {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{} {}]",
            self.label.0.iter().collect::<String>(),
            self.focal_length
        )
    }
}

impl Display for LensBoxes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, box_) in self
            .boxes
            .iter()
            .enumerate()
            .filter(|(_, b)| !b.lenses.is_empty())
        {
            writeln!(f, "Box {}:{}", i, box_)?;
        }
        Ok(())
    }
}

impl Debug for Lens {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Debug for AsciiString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().collect::<String>())
    }
}

impl Debug for LensBoxes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, box_) in self
            .boxes
            .iter()
            .enumerate()
            .filter(|(_, b)| !b.lenses.is_empty())
        {
            writeln!(f, "Box {}:{:?} | {:?}", i, box_.lenses, box_.indices)?;
        }
        Ok(())
    }
}
