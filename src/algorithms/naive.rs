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
        MuFormula::Bool(b) => { 
            let hs = HashSet::new();
            if b {
                return Ok(k.states.clone());
            }
            return Ok(hs);
        },
        MuFormula::Not(f) => {
            let result = try!(eval(&k, *f, e));
            return Ok(k.states.difference(&result).cloned().collect());
        },
        MuFormula::And(f, g) => {
            let left = try!(eval(&k, *f, e));
            let right = try!(eval(&k, *g, e));
            return Ok(left.intersection(&right).cloned().collect());
        },
        MuFormula::Or(f, g) => {
            let left = try!(eval(&k, *f, e));
            let right = try!(eval(&k, *g, e));
            return Ok(left.union(&right).cloned().collect());
        },

        // CTL
        MuFormula::Action(a) => { 
            //TODO: implement label function
            Ok(HashSet::new())
        },
        MuFormula::DiamondOp ( ac, f) => { 
            let states = try!(eval(&k, *f, e));
            let mut result = HashSet::new();
            for pat in &k.relations {
                if states.contains(&pat.2) && pat.1 == ac {
                    result.insert(pat.0);
                }
            }
            return Ok(result);
        },
        MuFormula::BoxOp (ac, f) => { 
            eval(k, MuFormula::Not(box MuFormula::DiamondOp(ac, f)), e)
        },

        // mu calculus
        MuFormula::RecursionValue(c) => { 
            let r = e.map.get(&c).map(|r| (*r).clone()).ok_or(MuErrors::VarNotFound(c));
            // println!("rec: {:?}", r);
            return r;
        },
        // least fixpoint operator
        MuFormula::Mu(c, f) => {
            e.map.insert(c.clone(), k.states.clone());
            let mut states = k.states.clone();
            let mut nstates = try!(eval(&k, *f.clone(), e));
            let mut cstates = states.clone();

            let mut iterations = 0;
            while states != nstates {
                states = states.intersection(&nstates).cloned().collect();
                // e.map.remove(&(c.clone()));
                e.map.insert((c.clone()), states.clone());
                nstates = try!(eval(&k, *f.clone(), e));
                if cstates == nstates && states != nstates {
                    panic!("mu formula failed, states are the same! after {} iterations", iterations);
                } else {
                    cstates = nstates.clone();
                    iterations+=1;
                }
            }
            return Ok(states);
        },
        // greatest fixpoint operator
        MuFormula::Nu(c, f) => {
            // let mut base = HashSet::<S>::new();
            // into variable, insert all states
            e.map.insert(c.clone(), HashSet::<S>::new());
            let mut states = HashSet::<S>::new();//try!(e.map.get(&(c.clone())).map(|r| (*r).clone()).ok_or(MuErrors::VarNotFound(c.clone())));
            // 
            let mut nstates = try!(eval(&k, *f.clone(), e));
            // println!("nu {:?} become {:?}", nstates, states);
            let mut cstates = states.clone();
            let mut iterations = 0;
            while states != nstates {
                states = states.union(&nstates).cloned().collect();
                e.map.insert((c.clone()), states.clone());
                nstates = try!(eval(&k, *f.clone(), e));
                if cstates == nstates  && states != nstates {
                    panic!("nu formula failed, states are the same! after {} iterations", iterations);
                } else {
                    cstates = nstates.clone();
                    iterations+=1;
                }
            }
            return Ok(states);
        }
    };
}