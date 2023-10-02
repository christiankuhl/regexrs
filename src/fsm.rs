use core::ops::{Index, IndexMut};
use std::fmt::Display;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum State {
    Failed,
    Intermediate(usize),
    Success,
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Failed => "F".to_owned(),
            Self::Success => "S".to_owned(),
            Self::Intermediate(x) => format!("{x}"),
        };
        write!(f, "{}", format!("({s})"))
    }
}

pub struct Transitions([State; 256]);

impl Default for Transitions {
    fn default() -> Self {
        Self([State::Failed; 256])
    }
}

impl Index<usize> for Transitions {
    type Output = State;
    fn index(&self, idx: usize) -> &Self::Output {
        &self.0[idx]
    }
}

impl IndexMut<usize> for Transitions {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        &mut self.0[idx]
    }
}

pub struct FSM {
    graph: Vec<Transitions>,
}

impl FSM {
    pub fn new() -> Self {
        Self { graph: Vec::new() }
    }
    pub fn final_state(&self) -> usize {
        self.graph.len()
    }
    pub fn push(&mut self, ts: Transitions) {
        self.graph.push(ts);
    }
    pub fn next(&self, state: State, char: char) -> State {
        match state {
            State::Failed => State::Failed,
            State::Success => State::Success,
            State::Intermediate(idx) => {
                if idx == self.final_state() {
                    return State::Success;
                }
                let nxt = self.graph[idx][char as usize];
                if let State::Intermediate(n) = nxt {
                    if n == self.final_state() {
                        return State::Success;
                    }
                }
                nxt
            }
        }
    }
}

impl Display for FSM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lines: Vec<String> = (0..=255usize)
            .filter(|&j| self.graph.iter().map(|v| v[j]).any(|x| x != State::Failed))
            .map(|j| {
                format!(
                    "{j:03} => {ts}",
                    ts = self
                        .graph
                        .iter()
                        .map(|v| format!("{:>5}", v[j]))
                        .collect::<Vec<String>>()
                        .join(" ")
                )
            })
            .collect();
        write!(f, "{}", lines.join("\n"))
    }
}
