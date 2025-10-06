use crate::common::day_setup::Day;
use crate::common::helpers::least_common_multiple_for;
use anyhow::{anyhow, Context as AnyhowContext};
use itertools::Itertools;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::iter::Sum;
use std::ops::{AddAssign, MulAssign};
use std::str::FromStr;
use strum_macros::EnumString;
pub fn day() -> Day {
    Day::new(run).with_test_inputs(&["broadcaster -> a
%a -> inv, con
&inv -> b
%b -> con
&con -> output"])
}

pub fn run(input: &str) {
    let times = 1_000;

    let mut network: Network = input.parse().unwrap();
    network.push_buttons(times);

    let pulses = network.history.pulses_after(times);
    println!(
        "pulses after {} times: {:?} => {}",
        times,
        pulses,
        pulses.high_pulses * pulses.low_pulses
    );

    let network: Network = input.parse().unwrap();

    let (dependency1, dependencies) = network.get_second_dependencies("rx");
    println!("dependency: {} -> {:?}", dependency1, dependencies);
    let presses: Vec<_> = dependencies
        .iter()
        .map(|dependency2| network.first_message_from_to(Pulse::High, dependency2, &dependency1))
        .collect();

    let rx_after = least_common_multiple_for(&presses);
    println!("least common multiple of {:?} = {}", presses, rx_after);
    println!("rx receives low after {} presses", rx_after);
}

#[derive(Clone)]
struct Network {
    name_to_index: HashMap<String, usize>,
    names: Vec<String>,
    outputs_by_module: Vec<Vec<usize>>,
    state: NetworkState,
    history: StateHistory,
}

impl Network {
    pub fn new(
        names: HashMap<String, usize>,
        outputs_by_module: Vec<Vec<usize>>,
        modules: Vec<Module>,
    ) -> Self {
        Self {
            names: names
                .iter()
                .sorted_by(|(_, a), (_, b)| a.cmp(b))
                .map(|(name, _)| name.clone())
                .collect(),
            name_to_index: names,
            outputs_by_module,
            state: NetworkState { modules },
            history: StateHistory::default(),
        }
    }
    pub fn get_second_dependencies(&self, module: &str) -> (String, Vec<String>) {
        let module = self.name_to_index[module];
        let [dependency1] = self
            .outputs_by_module
            .iter()
            .enumerate()
            .filter(|(_, outputs)| outputs.contains(&module))
            .map(|(i, _)| i)
            .collect::<Vec<_>>()[..]
        else {
            panic!("cannot find dependency1");
        };

        if let Module::Conjunction(inputs) = &self.state.modules[dependency1] {
            let dependencies2: Vec<_> = inputs.keys().map(|&key| self.names[key].clone()).collect();
            (self.names[dependency1].clone(), dependencies2)
        } else {
            panic!("dependency type is not what was expected");
        }
    }
    pub fn first_message_from_to(&self, message: Pulse, from: &str, to: &str) -> u64 {
        let message = Message {
            from: self.name_to_index[from],
            to: self.name_to_index[to],
            pulse: message,
        };
        let mut this = self.clone();
        let mut i = 1;
        while this.history.keep_going(&this.state) {
            let old_state = this.state.clone();
            if this.push_button_find_message(&message) {
                return i;
            }
            this.history.insert(old_state, PulseInfo::default());
            i += 1;
        }
        panic!("could not find message");
    }
    pub fn push_buttons(&mut self, max: u64) {
        let mut i = 0;
        while i < max && self.history.keep_going(&self.state) {
            let old_state = self.state.clone();
            let pulses = self.push_button();
            self.history.insert(old_state, pulses);
            i += 1;
        }
    }
    fn push_button(&mut self) -> PulseInfo {
        let mut pulses = PulseInfo::default();

        let mut messages = VecDeque::from([Message {
            from: self.name_to_index["button"],
            to: self.name_to_index["broadcaster"],
            pulse: Pulse::Low,
        }]);

        while let Some(message) = messages.pop_front() {
            pulses.add_pulse(message.pulse);
            log::debug!(
                "{} {:?} -> {}",
                self.names[message.from],
                message.pulse,
                self.names[message.to]
            );
            self.send_message(&message, &mut messages);
        }

        pulses
    }
    fn push_button_find_message(&mut self, target: &Message) -> bool {
        let mut messages = VecDeque::from([Message {
            from: self.name_to_index["button"],
            to: self.name_to_index["broadcaster"],
            pulse: Pulse::Low,
        }]);

        while let Some(message) = messages.pop_front() {
            if &message == target {
                return true;
            }

            log::trace!(
                "{} {:?} -> {}",
                self.names[message.from],
                message.pulse,
                self.names[message.to]
            );
            self.send_message(&message, &mut messages);
        }

        false
    }
    fn send_message(
        &mut self,
        message: &Message,
        output_message_collector: &mut VecDeque<Message>,
    ) {
        if let Some(output) = self.state.send_message(message) {
            let from = message.to;
            for &to in self.outputs_by_module[from].iter() {
                output_message_collector.push_back(Message {
                    from,
                    to,
                    pulse: output,
                })
            }
        }
    }
}

#[derive(Default, Clone)]
struct StateHistory {
    map: HashMap<NetworkState, usize>,
    pulses: Vec<PulseInfo>,
    loop_at: Option<usize>,
}

impl StateHistory {
    pub fn keep_going(&mut self, state: &NetworkState) -> bool {
        if let Some(&index) = self.map.get(state) {
            self.loop_at = Some(index);
            log::debug!("looped {} -> {}", index, self.pulses.len() - 1);

            false
        } else {
            true
        }
    }
    pub fn insert(&mut self, old_state: NetworkState, pulses: PulseInfo) {
        if self.map.contains_key(&old_state) {
            panic!("repeating state");
        }
        let index = self.pulses.len();
        self.pulses.push(pulses);
        self.map.insert(old_state, index);
    }
    pub fn pulses_after(&self, times: u64) -> PulseInfo {
        let index = self.loop_at.unwrap_or(usize::MAX);

        if (times as usize) < index {
            return self.pulses[0..(times as usize)].iter().cloned().sum();
        }

        let mut sum: PulseInfo = self.pulses[0..index].iter().cloned().sum();
        let times = times - index as u64;

        let mut chunk_sum: PulseInfo = self.pulses[index..].iter().cloned().sum();
        chunk_sum *= times / (self.pulses.len() - index) as u64;

        sum += chunk_sum;

        sum += self.pulses[index..(times as usize % (self.pulses.len() - index))]
            .iter()
            .cloned()
            .sum();

        sum
    }
}

#[derive(Eq, PartialEq, Hash, Clone)]
struct NetworkState {
    modules: Vec<Module>,
}

impl NetworkState {
    pub fn send_message(&mut self, message: &Message) -> Option<Pulse> {
        self.modules[message.to].process(message.pulse, message.from)
    }
}

#[derive(Debug, Default, Clone)]
struct PulseInfo {
    high_pulses: u64,
    low_pulses: u64,
}

impl PulseInfo {
    pub fn add_pulse(&mut self, pulse: Pulse) {
        match pulse {
            Pulse::Low => {
                self.low_pulses += 1;
            }
            Pulse::High => {
                self.high_pulses += 1;
            }
        }
    }
}

#[derive(Eq, PartialEq)]
struct Message {
    from: usize,
    to: usize,
    pulse: Pulse,
}

#[derive(Debug, strum_macros::Display, EnumString, Clone, Eq, PartialEq, Hash)]
enum Module {
    #[strum(serialize = "broadcaster")]
    Broadcaster,
    #[strum(serialize = "%")]
    FlipFlop(Pulse),
    #[strum(serialize = "&")]
    Conjunction(BTreeMap<usize, Pulse>),
    #[strum(serialize = "noop")]
    NoOp,
}

impl Module {
    pub fn process(&mut self, input: Pulse, from: usize) -> Option<Pulse> {
        match self {
            Module::FlipFlop(state) => {
                if input == Pulse::Low {
                    *state = state.inverse();
                    Some(*state)
                } else {
                    None
                }
            }
            Module::Conjunction(state) => {
                *state.get_mut(&from).expect("module inputs set incorrectly") = input;
                Some(if state.values().all(|&val| val == Pulse::High) {
                    Pulse::Low
                } else {
                    Pulse::High
                })
            }
            Module::Broadcaster => Some(input),
            Module::NoOp => None,
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
enum Pulse {
    #[default]
    Low,
    High,
}

impl Pulse {
    pub fn inverse(&self) -> Self {
        match self {
            Pulse::Low => Pulse::High,
            Pulse::High => Pulse::Low,
        }
    }
}

impl AddAssign for PulseInfo {
    fn add_assign(&mut self, rhs: Self) {
        self.high_pulses += rhs.high_pulses;
        self.low_pulses += rhs.low_pulses;
    }
}

impl Sum for PulseInfo {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut result = Self::default();
        for other in iter {
            result += other;
        }
        result
    }
}

impl MulAssign<u64> for PulseInfo {
    fn mul_assign(&mut self, rhs: u64) {
        self.high_pulses *= rhs;
        self.low_pulses *= rhs;
    }
}

impl FromStr for Network {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut names: HashMap<String, usize> = Default::default();
        let mut outputs_by_module: Vec<Vec<usize>> = Default::default();
        let mut modules: Vec<Module> = Default::default();
        let mut outputs_by_name: HashMap<&str, Vec<&str>> = Default::default();
        let mut inputs_by_name: HashMap<&str, Vec<&str>> = Default::default();

        let mut insert = |module: Module, name: String| {
            let index = modules.len();
            modules.push(module);
            names.insert(name, index);
            outputs_by_module.push(Default::default());
        };
        insert(Module::NoOp, "output".to_string());
        insert(Module::NoOp, "rx".to_string());
        insert(Module::NoOp, "button".to_string());

        for line in s.lines() {
            let [from, tos] = line.split("->").collect::<Vec<_>>()[..] else {
                return Err(anyhow!("invalid line '{}'", line));
            };
            let from = from.trim();

            let (from, module, name) = if let Some(from) = from.strip_prefix("%") {
                (from, Module::FlipFlop(Pulse::Low), from.to_string())
            } else if let Some(from) = from.strip_prefix("&") {
                (
                    from,
                    Module::Conjunction(Default::default()),
                    from.to_string(),
                )
            } else if from == "broadcaster" {
                (from, Module::Broadcaster, from.to_string())
            } else {
                return Err(anyhow!("invalid module name '{}'", from));
            };

            insert(module, name);

            let outputs = outputs_by_name.entry(from).or_default();
            for to in tos.split(",") {
                let to = to.trim();
                outputs.push(to);
                inputs_by_name.entry(to).or_default().push(from);
            }
        }

        for (from, tos) in outputs_by_name {
            let from = *names
                .get(from)
                .with_context(|| format!("from name '{}' not set", from))?;
            for to in tos {
                let to = *names
                    .get(to)
                    .with_context(|| format!("to name '{}' not set", to))?;
                outputs_by_module[from].push(to);
            }
        }
        for (to, from) in inputs_by_name {
            let to = *names
                .get(to)
                .with_context(|| format!("to name '{}' not set 2", to))?;
            if let Module::Conjunction(inputs) = &mut modules[to] {
                for from in from {
                    let from = *names
                        .get(from)
                        .with_context(|| format!("from name '{}' not set 2", from))?;
                    inputs.insert(from, Pulse::Low);
                }
            }
        }

        Ok(Self::new(names, outputs_by_module, modules))
    }
}
