use ddo::core::abstraction::dp::Problem;
use ddo::core::common::{Decision, Domain, Variable, VarSet};
use std::cmp::{max, min, Ordering};
use std::fs::File;
use std::io::{BufRead, BufReader, Lines, Read};
use std::ops::{Index, IndexMut};

use crate::instance::Weighed2Sat;

const T  : i32      = 1;
const F  : i32      =-1;
const TF : [i32; 2] = [T, F];

const fn v (x: Variable) -> i32 { 1 + x.0 as i32}
const fn t (x: Variable) -> i32 { v(x) }
const fn f (x: Variable) -> i32 {-v(x) }
fn pos(x: i32) -> i32 { max(0, x) }


#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct State {
    pub substates: Vec<i32>
}

impl Index<Variable> for State {
    type Output = i32;

    fn index(&self, index: Variable) -> &i32 {
        self.substates.get(index.0).unwrap()
    }
}
impl IndexMut<Variable> for State {
    fn index_mut(&mut self, index: Variable) -> &mut i32 {
        self.substates.get_mut(index.0).unwrap()
    }
}
impl State {
    pub fn rank(&self) -> i32 {
        self.substates.iter().map(|x| x.abs()).sum()
    }
}
impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank().cmp(&other.rank())
    }
}
impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
pub struct Max2Sat {
    pub nb_vars : usize,
    pub initial : i32,
    pub weights : Vec<i32>,
    pub sum_of_clause_weights: Vec<i32>
}

const fn idx(x: i32) -> usize {
    (x.abs() - 1) as usize
}
fn mk_lit(x: i32) -> usize {
    let sign = if x > 0 { 1 } else { 0 };
    let abs  = (x.abs() - 1) as usize;

    abs + abs + sign
}
impl Max2Sat {
    pub fn new(inst: Weighed2Sat) -> Max2Sat {
        let n = inst.nb_vars;
        let mut ret = Max2Sat{
            nb_vars: n,
            initial: 0,
            weights: vec![0; (2*n)*(2*n)],
            sum_of_clause_weights: vec![0; n]
        };

        for (clause, weight) in inst.weights.iter() {
            let of = ret.offset(clause.a, clause.b);
            ret.weights[of] = *weight;

            ret.sum_of_clause_weights[idx(clause.a)] += *weight;

            if !clause.is_unit() {
                ret.sum_of_clause_weights[idx(clause.b)] += *weight;
            }
            if clause.is_tautology() {
                ret.initial += *weight;
            }
        }
        ret
    }

    pub fn weight(&self, x: i32, y: i32) -> i32 {
        self.weights[self.offset(x, y)]
    }

    fn offset(&self, x: i32, y: i32) -> usize {
        let a = x.min(y);
        let b = x.max(y);

        (mk_lit(a) * 2 * self.nb_vars) + mk_lit(b)
    }
}

impl Problem<State> for Max2Sat {

    fn nb_vars(&self) -> usize {
        self.nb_vars
    }

    fn initial_state(&self) -> State {
        State{substates: vec![0; self.nb_vars()]}
    }

    fn initial_value(&self) -> i32 {
        // sum of all tautologies
        self.initial
    }

    fn domain_of<'a>(&self, _state: &'a State, _var: Variable) -> Domain<'a> {
        Domain::Slice(&TF)
    }

    fn transition(&self, state: &State, vars: &VarSet, d: Decision) -> State {
        let k = d.variable;
        let mut ret  = state.clone();
        ret[k] = 0;
        if d.value == F {
            for l in vars.iter() {
                ret[l] += self.weight(t(k), t(l)) - self.weight(t(k), f(l));
            }
        } else {
            for l in vars.iter() {
                ret[l] += self.weight(f(k), t(l)) - self.weight(f(k), f(l));
            }
        }
        ret
    }

    fn transition_cost(&self, state: &State, vars: &VarSet, d: Decision) -> i32 {
        let k = d.variable;
        if d.value == F {
            let res = pos(-state[k]);
            let mut sum = self.weight(f(k), f(k)); // Weight if unit clause
            for l in vars.iter() {
                // Those that are satisfied by [[ k = F ]]
                let wff = self.weight(f(k), f(l));
                let wft = self.weight(f(k), t(l));
                // Those that actually depend on the truth value of `l`.
                let wtt = self.weight(t(k), t(l));
                let wtf = self.weight(t(k), f(l));

                sum += (wff + wft) + min(pos( state[l]) + wtt,
                                         pos(-state[l]) + wtf);
            }

            res + sum
        } else /*if d.value == T*/ {
            let res = pos(state[k]);
            let mut sum = self.weight(t(k), t(k)); // Weight if unit clause
            for l in vars.iter() {
                // Those that are satisfied by [[ k = T ]]
                let wtt = self.weight(t(k), t(l));
                let wtf = self.weight(t(k), f(l));
                // Those that actually depend on the truth value of `l`.
                let wff = self.weight(f(k), f(l));
                let wft = self.weight(f(k), t(l));

                sum += (wtf + wtt) + min(pos( state[l]) + wft,
                                         pos(-state[l]) + wff);
            }

            res + sum
        }
    }
}
impl From<File> for Max2Sat {
    fn from(file: File) -> Self {
        BufReader::new(file).into()
    }
}
impl <S: Read> From<BufReader<S>> for Max2Sat {
    fn from(buf: BufReader<S>) -> Self {
        buf.lines().into()
    }
}
impl <B: BufRead> From<Lines<B>> for Max2Sat {
    fn from(lines: Lines<B>) -> Self {
        Max2Sat::new(lines.into())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;
    use std::fs::File;

    #[test]
    fn test_index_state() {
        let state = State{substates: vec![1, 2, 3, 4]};

        assert_eq!(state[Variable(0)], 1);
        assert_eq!(state[Variable(1)], 2);
        assert_eq!(state[Variable(2)], 3);
        assert_eq!(state[Variable(3)], 4);
    }
    #[test]
    fn test_index_mut_state() {
        let mut state = State{substates: vec![1, 2, 3, 4]};

        state[Variable(0)] = 42;
        state[Variable(1)] = 64;
        state[Variable(2)] = 16;
        state[Variable(3)] =  9;

        assert_eq!(state[Variable(0)], 42);
        assert_eq!(state[Variable(1)], 64);
        assert_eq!(state[Variable(2)], 16);
        assert_eq!(state[Variable(3)],  9);
    }

    #[test]
    fn test_initial_value() {
        let id         = "debug2.wcnf";
        let problem    = instance(id);

        assert_eq!(0, problem.initial_value());
    }

    #[test]
    fn test_next_state() {
        let id         = "debug2.wcnf";
        let problem    = instance(id);

        let mut vars   = problem.all_vars();
        let root       = problem.root_node();
        let expected = State{substates: vec![0, 0, 0]};
        assert_eq!(expected, root.state);

        vars.remove(Variable(0));
        let dec_f      = Decision{variable: Variable(0), value: F};
        let nod_f      = problem.transition(&root.state, &vars, dec_f);

        let expected = State{substates: vec![0,-4, 3]};
        assert_eq!(expected, nod_f);

        let dec_t      = Decision{variable: Variable(0), value: 1};
        let nod_t     = problem.transition(&root.state, &vars, dec_t);
        let expected = State{substates: vec![0, 0, 0]};
        assert_eq!(expected, nod_t);
    }

    #[test]
    fn test_rank() {
        let benef =
        vec![-183, -122, -61, -183, -183, -183, -122, -122, -183, -122, -61, -122,
             0, -122, -122, -122, -122, -122, -183, -122, -61, 0, -122, -61, 0, 0,
             0, 0, 0, -244, -61, -183, 0, -122, -244, -183, -61, -61, -122, -122,
             -122, -183, -122, 0, -183, -61, -183, -122, -122, -183, -183, -61,
             -61, -122, 0, 0, 0, 0, 0, 0];
        let state = State{substates: benef};

        assert_eq!(5917, state.rank());
    }

    fn locate(id: &str) -> PathBuf {
        PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("examples/tests/resources/max2sat/")
            .join(id)
    }

    fn instance(id: &str) -> Max2Sat {
        let location = locate(id);
        File::open(location).expect("File not found").into()
    }
}