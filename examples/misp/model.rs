use std::fs::File;
use std::io::{BufRead, BufReader, Lines, Read};
use std::ops::Not;

use bitset_fixed::BitSet;

use ddo::core::abstraction::dp::Problem;
use ddo::core::common::{Decision, Variable, VarSet, Domain};

use crate::instance::Graph;

#[derive(Debug, Clone)]
pub struct Misp {
    pub graph : Graph
}

impl Misp {
    pub fn new(mut graph : Graph) -> Misp {
        graph.complement();
        Misp {graph}
    }
}

const YES_NO : [i32; 2] = [1, 0];
const NO    : [i32; 1] = [0];

impl Problem<BitSet> for Misp {
    fn nb_vars(&self) -> usize {
        self.graph.nb_vars
    }

    fn initial_state(&self) -> BitSet {
        BitSet::new(self.graph.nb_vars).not()
    }

    fn initial_value(&self) -> i32 {
        0
    }

    fn domain_of<'a>(&self, state: &'a BitSet, var: Variable) -> Domain<'a> {
        if state[var.0] { Domain::Slice(&YES_NO) } else { Domain::Slice(&NO) }
    }

    fn transition(&self, state: &BitSet, _vars: &VarSet, d: Decision) -> BitSet {
        let mut bs = state.clone();
        bs.set(d.variable.0, false);

        // drop adjacent vertices if needed
        if d.value == 1 {
            bs &= &self.graph.adj_matrix[d.variable.0];
        }

        bs
    }

    fn transition_cost(&self, _state: &BitSet, _vars: &VarSet, d: Decision) -> i32 {
        if d.value == 0 {
            0
        } else {
            self.graph.weights[d.variable.0]
        }
    }

    fn impacted_by(&self, state: &BitSet, variable: Variable) -> bool {
        state[variable.0]
    }
}
impl From<File> for Misp {
    fn from(file: File) -> Self {
        BufReader::new(file).into()
    }
}
impl <S: Read> From<BufReader<S>> for Misp {
    fn from(buf: BufReader<S>) -> Self {
        buf.lines().into()
    }
}
impl <B: BufRead> From<Lines<B>> for Misp {
    fn from(lines: Lines<B>) -> Self {
        Self::new(lines.into())
    }
}