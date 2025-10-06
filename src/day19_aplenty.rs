use crate::common::day_setup::Day;
use anyhow::Context as AnyhowContext;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use std::borrow::Cow;
use std::collections::{BTreeSet, HashMap};
use std::ops::RangeInclusive;
use std::str::FromStr;
pub fn day() -> Day {
    Day::new(run).with_test_inputs(&["px{a<2006:qkq,m>2090:A,rfg}
pv{a>1716:R,A}
lnx{m>1548:A,A}
rfg{s<537:gd,x>2440:R,A}
qs{s>3448:A,lnx}
qkq{x<1416:A,crn}
crn{x>2662:A,R}
in{s<1351:px,qqz}
qqz{s>2770:qs,m<1801:hdj,R}
gd{a>3333:R,R}
hdj{m>838:A,pv}

{x=787,m=2655,a=1222,s=2876}
{x=1679,m=44,a=2067,s=496}
{x=2036,m=264,a=79,s=2244}
{x=2461,m=1339,a=466,s=291}
{x=2127,m=1623,a=2188,s=1013}"])
}

pub fn run(input: &str) {
    let mut inputs = input.split("\n\n");

    let workflows: HashMap<Cow<'static, str>, Workflow> = inputs
        .next()
        .unwrap()
        .lines()
        .map(|line| line.parse::<Workflow>().unwrap())
        .map(|workflow| (workflow.name.clone().into(), workflow))
        .collect();

    let parts: Vec<Part> = inputs
        .next()
        .unwrap()
        .lines()
        .map(|line| line.parse().unwrap())
        .collect();

    let sum: u64 = parts
        .iter()
        .filter(|part| part.is_accepted(&workflows))
        .map(|part| part.sum_ratings())
        .sum();
    println!("part 1 sum: {}", sum);

    let combinations = Workflows::new(workflows).combinations_of_ratings_accepted(1..=4_000);
    println!("part 2 total combinations: {}", combinations);
}

struct PartWorkflows<'a> {
    part: &'a Part,
    workflows: &'a HashMap<Cow<'static, str>, Workflow>,
}

impl<'a> PartWorkflows<'a> {
    pub fn new(part: &'a Part, workflows: &'a HashMap<Cow<'static, str>, Workflow>) -> Self {
        Self { part, workflows }
    }
    pub fn is_accepted(&self) -> bool {
        let mut workflow = Cow::Borrowed("in");
        loop {
            let next = self.workflows.get(&workflow).unwrap();
            match next.follow(self.part) {
                Destination::Accept => {
                    return true;
                }
                Destination::Reject => {
                    return false;
                }
                Destination::Workflow(next) => {
                    workflow = next;
                }
            }
        }
    }
}

struct Workflows {
    workflows: HashMap<Cow<'static, str>, Workflow>,
}

impl Workflows {
    pub fn new(mut workflows: HashMap<Cow<'static, str>, Workflow>) -> Self {
        for workflow in workflows.values_mut() {
            workflow.simplify();
        }
        Self { workflows }
    }
    pub fn combinations_of_ratings_accepted(&self, range: RangeInclusive<u64>) -> u64 {
        let paths: HashMap<Category, BTreeSet<u64>> = self
            .workflows
            .values()
            .map(|workflow| workflow.get_ratings_paths())
            .fold(HashMap::default(), |mut acc, next| {
                for next in next {
                    let category = acc.entry(next.category).or_default();
                    category.insert(next.left);
                    category.insert(next.right);
                }
                acc
            });

        let paths_and_counts: HashMap<Category, Vec<(u64, u64)>> = paths
            .into_iter()
            .map(|(category, mut paths)| {
                paths.insert(*range.start());
                paths.insert(*range.end() + 1);
                let paths: Vec<_> = paths.into_iter().collect();
                let paths_counts: Vec<_> = paths
                    .windows(2)
                    .map(|window| {
                        let start = window[0];
                        let end_exclusive = window[1];
                        let count = end_exclusive - start;
                        (start, count)
                    })
                    .collect();

                (category, paths_counts)
            })
            .collect();

        for (category, paths_and_counts) in paths_and_counts.iter() {
            println!("{}: {} possibilities", category, paths_and_counts.len());
        }

        paths_and_counts
            .get(&Category::ExtremelyCoolLooking)
            .unwrap()
            .par_iter()
            .map(|x| {
                let mut count_passed = 0;
                for m in paths_and_counts.get(&Category::Musical).unwrap() {
                    for a in paths_and_counts.get(&Category::Aerodynamic).unwrap() {
                        for s in paths_and_counts.get(&Category::Shiny).unwrap() {
                            let part = Part {
                                ratings: HashMap::from([
                                    (Category::ExtremelyCoolLooking, x.0),
                                    (Category::Musical, m.0),
                                    (Category::Aerodynamic, a.0),
                                    (Category::Shiny, s.0),
                                ]),
                            };

                            if part.is_accepted(&self.workflows) {
                                count_passed += x.1 * m.1 * a.1 * s.1;
                            }
                        }
                    }
                }

                count_passed
            })
            .sum()
    }
}

struct Part {
    ratings: HashMap<Category, u64>,
}

impl Part {
    fn try_from_iter<E: Into<anyhow::Error>>(
        ratings: impl Iterator<Item = Result<(Category, u64), E>>,
    ) -> Result<Self, E> {
        let ratings: HashMap<_, _> = ratings.collect::<Result<_, _>>()?;
        assert!(
            ratings.contains_key(&Category::ExtremelyCoolLooking)
                && ratings.contains_key(&Category::Musical)
                && ratings.contains_key(&Category::Aerodynamic)
                && ratings.contains_key(&Category::Shiny),
            "part doesn't contain all ratings"
        );
        Ok(Self { ratings })
    }
    pub fn rating(&self, category: Category) -> u64 {
        *self.ratings.get(&category).unwrap()
    }
    pub fn sum_ratings(&self) -> u64 {
        self.ratings.values().sum()
    }
    pub fn is_accepted(&self, workflows: &HashMap<Cow<'static, str>, Workflow>) -> bool {
        PartWorkflows::new(self, workflows).is_accepted()
    }
}

struct Workflow {
    name: String,
    rules: Vec<Rule>,
    fallback: Destination<'static>,
}

impl Workflow {
    pub fn follow(&self, part: &Part) -> Destination<'_> {
        for rule in self.rules.iter() {
            if let Some(destination) = rule.try_follow(part) {
                return destination;
            }
        }

        self.fallback.borrowed()
    }
    pub fn simplify(&mut self) {
        loop {
            if self.rules.last().map(|rule| &rule.destination) == Some(&self.fallback) {
                let rule = self.rules.pop();
                log::debug!("removed rule {:?}", rule);
            } else {
                break;
            }
        }
    }
    pub fn get_ratings_paths(&self) -> Vec<RatingNumberPaths> {
        self.rules
            .iter()
            .map(|rule| rule.get_rating_paths())
            .collect()
    }
}

struct RatingNumberPaths {
    category: Category,
    left: u64,
    right: u64,
}

#[derive(Eq, PartialEq, Debug)]
enum Destination<'a> {
    Accept,
    Reject,
    Workflow(Cow<'a, str>),
}

impl<'a> Destination<'a> {
    pub fn borrowed<'b>(&'a self) -> Destination<'b>
    where
        'a: 'b,
    {
        match self {
            Destination::Accept => Destination::Accept,
            Destination::Reject => Destination::Reject,
            Destination::Workflow(val) => Destination::Workflow(Cow::Borrowed(val)),
        }
    }
}

#[derive(Debug)]
struct Rule {
    operation: Operation,
    category: Category,
    check_value: u64,
    destination: Destination<'static>,
}

impl Rule {
    pub fn try_follow(&self, part: &Part) -> Option<Destination<'_>> {
        if self
            .operation
            .operate(part.rating(self.category), self.check_value)
        {
            Some(self.destination.borrowed())
        } else {
            None
        }
    }
    pub fn get_rating_paths(&self) -> RatingNumberPaths {
        match self.operation {
            Operation::LessThan => RatingNumberPaths {
                category: self.category,
                left: self.check_value - 1,
                right: self.check_value,
            },
            Operation::GreaterThan => RatingNumberPaths {
                category: self.category,
                left: self.check_value,
                right: self.check_value + 1,
            },
        }
    }
}

#[derive(
    strum_macros::Display, strum_macros::EnumString, Copy, Clone, Eq, PartialEq, Hash, Debug,
)]
enum Category {
    #[strum(serialize = "x")]
    ExtremelyCoolLooking,
    #[strum(serialize = "m")]
    Musical,
    #[strum(serialize = "a")]
    Aerodynamic,
    #[strum(serialize = "s")]
    Shiny,
}

#[derive(strum_macros::Display, strum_macros::EnumString, Copy, Clone, Eq, PartialEq, Debug)]
enum Operation {
    #[strum(serialize = "<")]
    LessThan,
    #[strum(serialize = ">")]
    GreaterThan,
}

impl Operation {
    pub fn operate(&self, left: u64, right: u64) -> bool {
        match self {
            Operation::LessThan => left < right,
            Operation::GreaterThan => left > right,
        }
    }
}

impl FromStr for Workflow {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const BRACKET_PATTERN: &str = "{";
        let bracket = s.find(BRACKET_PATTERN).context("cannot find {")?;
        let name = s[..bracket].to_string();
        let s = s[bracket + BRACKET_PATTERN.len()..].trim_end_matches("}");
        let mut parts = s.split(",").collect::<Vec<_>>().into_iter().rev();
        let fallback: Destination = parts.next().context("empty body")?.into();
        let rules: Vec<Rule> = parts
            .map(|p| p.parse())
            .rev()
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            name,
            rules,
            fallback,
        })
    }
}

impl From<&str> for Destination<'static> {
    fn from(value: &str) -> Self {
        match value {
            "A" => Self::Accept,
            "R" => Self::Reject,
            other => Self::Workflow(other.to_string().into()),
        }
    }
}

impl FromStr for Rule {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let [s, destination] = s.split(":").collect::<Vec<_>>()[..] else {
            return Err(anyhow::anyhow!("rule missing ':': '{}'", s));
        };
        let destination: Destination = destination.into();

        let op = if s.contains(">") {
            ">"
        } else if s.contains("<") {
            "<"
        } else {
            return Err(anyhow::anyhow!("unknown rule operation '{}'", s));
        };
        let [category, check_value] = s.split(op).collect::<Vec<&str>>()[..] else {
            return Err(anyhow::anyhow!("invalid rule format '{}'", s));
        };
        let category: Category = category.parse()?;
        let check_value: u64 = check_value.parse()?;
        Ok(Self {
            operation: op.parse()?,
            category,
            check_value,
            destination,
        })
    }
}

impl FromStr for Part {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from_iter(s.trim_matches(['{', '}']).split(",").map(|category| {
            let [category, value] = category.split("=").collect::<Vec<_>>()[..] else {
                return Err(anyhow::anyhow!("invalid category '{}'", category));
            };
            let category: Category = category.parse()?;
            let value: u64 = value.parse()?;
            Ok((category, value))
        }))
    }
}
