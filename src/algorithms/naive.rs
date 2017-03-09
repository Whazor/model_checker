use parsers::mucalculus::MuFormula;
use parsers::kripke_structure::MixedKripkeStructure;
use std::collections::HashSet;
use std::collections::HashMap;
use std::hash::Hash;
use std::fmt::Debug;
use std::{thread, time};


struct Environment<'time, S: Hash+Eq+Clone+Copy+'time+Debug> {
    map: &'time mut HashMap<String, HashSet<S>>
}

pub fn evaluate<'time, S: Hash+Eq+Clone+Copy+'time+Debug, L: Clone+Copy>(k: &'time MixedKripkeStructure<S, L>, mu: MuFormula) -> Result<HashSet<S>, MuErrors> {
    let mut env = Environment {
        map: &mut HashMap::new()
     };
    return eval(k, mu, &mut env);
}

#[derive(Debug)]
pub enum MuErrors {
    VarNotFound(String)
}

fn eval<'time, S: Hash+Eq+Clone+Copy+'time+Debug, L: Clone+Copy>(k: &'time MixedKripkeStructure<S, L>, mu: MuFormula, e: &'time mut Environment<S>) -> Result<HashSet<S>, MuErrors> {
    return match mu.clone() {
        // logic
        MuFormula::Bool(_, b) => { 
            let hs = HashSet::new();
            if b {
                return Ok(k.states.clone());
            }
            return Ok(hs);
        },
        MuFormula::Not(_, f) => {
            let result = try!(eval(&k, *f, e));
            return Ok(k.states.difference(&result).cloned().collect());
        },
        MuFormula::And(_, f, g) => {
            let left = try!(eval(&k, *f, e));
            let right = try!(eval(&k, *g, e));
            return Ok(left.intersection(&right).cloned().collect());
        },
        MuFormula::Or(_, f, g) => {
            let left = try!(eval(&k, *f, e));
            let right = try!(eval(&k, *g, e));
            return Ok(left.union(&right).cloned().collect());
        },

        // CTL
        MuFormula::Action(_, a) => { 
            //TODO: implement label function
            Ok(HashSet::new())
        },
        MuFormula::DiamondOp (_, ac, f) => { 
            let states = try!(eval(&k, *f, e));
            let mut result = HashSet::new();
            for pat in &k.relations {
                if states.contains(&pat.2) && pat.1 == ac {
                    result.insert(pat.0);
                }
            }
            return Ok(result);
        },
        MuFormula::BoxOp (p, ac, f) => { 
            eval(k, MuFormula::Not(p, box MuFormula::DiamondOp(p, ac, f)), e)
        },

        // mu calculus
        MuFormula::RecursionValue(_, c) => { 
            let r = e.map.get(&c).map(|r| (*r).clone()).ok_or(MuErrors::VarNotFound(c));
            // println!("rec: {:?}", r);
            return r;
        },
        // least fixpoint operator
        MuFormula::Mu(_, c, f) => {
 
            let mut states = HashSet::<S>::new();
            let mut nstates = k.states.clone();
            let mut cstates = states.clone();
            let mut iterations = 0;
            loop {
                e.map.insert(c.clone(), nstates.clone());
                nstates = try!(eval(&k, *f.clone(), e));
                states = nstates.union(&try!(e.map.get(&(c.clone())).map(|r| (*r).clone()).ok_or(MuErrors::VarNotFound(c.clone())))).cloned().collect();
                if cstates == nstates && states != nstates && iterations > 100 {
                    panic!("mu formula failed, states are the same! after {} iterations", iterations);
                } else {
                    cstates = nstates.clone();
                    iterations+=1;
                }
                if states == nstates { 
                    break; 
                }
            }
            return Ok(states);
        },
        // greatest fixpoint operator
        MuFormula::Nu(_, c, f) => {
            // into variable, insert all states
            let mut states = HashSet::<S>::new();
            let mut nstates = HashSet::<S>::new();
            let mut cstates = states.clone();
            let mut iterations = 0;
            loop {
                e.map.insert(c.clone(), nstates.clone());
                nstates = try!(eval(&k, *f.clone(), e));
                states = nstates.intersection(&try!(e.map.get(&(c.clone())).map(|r| (*r).clone()).ok_or(MuErrors::VarNotFound(c.clone())))).cloned().collect();
                if cstates == nstates && states != nstates && iterations > 100 {
                    panic!("nu formula failed, states are the same! after {} iterations", iterations);
                } else {
                    cstates = nstates.clone();
                    iterations+=1;
                }
                if states == nstates { 
                    break; 
                }
            }
            return Ok(states);
        }
    };
}