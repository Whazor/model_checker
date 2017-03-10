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
    return eval(k, &mu, &mut env);
}

#[derive(Debug)]
pub enum MuErrors {
    VarNotFound(String)
}

fn eval<'time, S: Hash+Eq+Clone+Copy+'time+Debug, L: Clone+Copy>(k: &'time MixedKripkeStructure<S, L>, mu: &MuFormula, e: &'time mut Environment<S>) -> Result<HashSet<S>, MuErrors> {
    return match *mu {
        // logic
        MuFormula::Bool(_, b) => { 
            let hs = HashSet::new();
            if b {
                return Ok(k.states.clone());
            }
            return Ok(hs);
        },
        MuFormula::Not(_, ref f) => {
            let result = try!(eval(&k, f, e));
            return Ok(k.states.difference(&result).cloned().collect());
        },
        MuFormula::And(_, ref f, ref g) => {
            let left = try!(eval(&k, f, e));
            let right = try!(eval(&k, g, e));
            return Ok(left.intersection(&right).cloned().collect());
        },
        MuFormula::Or(_, ref f, ref g) => {
            let left = try!(eval(&k, f, e));
            let right = try!(eval(&k, g, e));
            return Ok(left.union(&right).cloned().collect());
        },
        // CTL
        MuFormula::Action(_, _) => { 
            //TODO: implement label function
            Ok(HashSet::new())
        },
        MuFormula::DiamondOp (p, ref ac, ref f) => { 
            eval(k, 
                &MuFormula::Not(p, box MuFormula::BoxOp(p, ac.clone(), box MuFormula::Not(p, box *f.clone()))),
            e)
        },
        MuFormula::BoxOp (_, ref ac, ref f) => { 
            let states = try!(eval(&k, f, e));
            let mut result = HashSet::new();
            for s in k.states.clone() {
                let mut insert = true;
                for pat in &k.relations {
                    if pat.0 == s && pat.1 == *ac {
                        if !(states.contains(&pat.2)) {
                            insert = false;
                        }
                    }
                }
                if insert {
                    result.insert(s);
                }
            }
            return Ok(result);
        },

        // mu calculus
        MuFormula::RecursionValue(_, ref c) => { 
            let r = e.map.get(c).map(|r| (*r).clone()).ok_or(MuErrors::VarNotFound(c.clone()));
            // println!("rec: {:?}", r);
            return r;
        },
        // least fixpoint operator
        MuFormula::Mu(_, ref c, ref f) => {
            let mut states = HashSet::<S>::new();
            let mut nstates = HashSet::<S>::new();//;k.states.clone();
            loop {
                e.map.insert(c.clone(), nstates.clone());
                nstates = try!(eval(&k, f, e));
                states = nstates.intersection(&try!(e.map.get(&(c.clone())).map(|r| (*r).clone()).ok_or(MuErrors::VarNotFound(c.clone())))).cloned().collect();
                if states == nstates { 
                    break; 
                }
            }
            return Ok(states);
        },
        // greatest fixpoint operator
        MuFormula::Nu(_, ref c, ref f) => {
            let mut states = HashSet::<S>::new();
            let mut nstates = k.states.clone();
            loop {
                e.map.insert(c.clone(), nstates.clone());
                nstates = try!(eval(&k, f, e));
                states = nstates.union(&try!(e.map.get(&(c.clone())).map(|r| (*r).clone()).ok_or(MuErrors::VarNotFound(c.clone())))).cloned().collect();
                if states == nstates { 
                    break; 
                }
            }
            return Ok(states);
        }
    };
}