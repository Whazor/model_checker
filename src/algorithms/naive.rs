use parsers::mucalculus::MuFormula;
use parsers::kripke_structure::MixedKripkeStructure;
use std::collections::HashSet;
use std::collections::HashMap;
use std::hash::Hash;
use std::fmt::Debug;
use std::{thread, time};
use bit_set::BitSet;

struct Environment<'time> {
    map: &'time mut HashMap<String, BitSet>
}

pub fn evaluate<'time, L: Clone+Copy>(k: &'time MixedKripkeStructure<L>, mu: MuFormula) -> Result<BitSet, MuErrors> {
    let mut env = Environment {
        map: &mut HashMap::new()
     };
    return eval(k, &mu, &mut env);
}

#[derive(Debug)]
pub enum MuErrors {
    VarNotFound(String)
}

fn eval<'time, L: Clone+Copy>(k: &'time MixedKripkeStructure<L>, mu: &MuFormula, e: &'time mut Environment) -> Result<BitSet, MuErrors> {
    return match *mu {
        // logic
        MuFormula::Bool(_, b) => { 
            let hs = BitSet::new();
            if b {
                return Ok(k.states.clone());
            }
            return Ok(hs);
        },
        MuFormula::Not(_, ref f) => {
            let result = try!(eval(&k, f, e));
            return Ok(k.states.difference(&result).collect::<BitSet>());
        },
        MuFormula::And(_, ref f, ref g) => {
            let left = try!(eval(&k, f, e));
            let right = try!(eval(&k, g, e));
            return Ok(left.intersection(&right).collect::<BitSet>());
        },
        MuFormula::Or(_, ref f, ref g) => {
            let left = try!(eval(&k, f, e));
            let right = try!(eval(&k, g, e));
            return Ok(left.union(&right).collect::<BitSet>());
        },
        // CTL
        MuFormula::Action(_, _) => { 
            //TODO: implement label function
            Ok(BitSet::new())
        },
        MuFormula::DiamondOp (p, ref ac, ref f) => { 
            eval(k, 
                &MuFormula::Not(p, box MuFormula::BoxOp(p, ac.clone(), box MuFormula::Not(p, box *f.clone()))),
            e)
        },
        MuFormula::BoxOp (_, ref ac, ref f) => { 
            let states = try!(eval(&k, f, e));
            let mut result = BitSet::new();
            for s in k.states.into_iter() {
                let mut insert = true;
                for pat in k.relations.get(&(s as u32, String::from(ac.clone()))).unwrap_or(&BitSet::new()) {
                    if !(states.contains(pat)) {
                        insert = false;
                    }
                }
                if insert {
                    result.insert(s as usize);
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
            let mut states = BitSet::new();
            let mut nstates = BitSet::new();
            loop {
                e.map.insert(c.clone(), nstates.clone());
                nstates = try!(eval(&k, f, e));
                states = nstates.intersection(&try!(e.map.get(&(c.clone())).map(|r| (*r).clone()).ok_or(MuErrors::VarNotFound(c.clone())))).collect::<BitSet>();
                if states == nstates { 
                    break; 
                }
            }
            return Ok(states);
        },
        // greatest fixpoint operator
        MuFormula::Nu(_, ref c, ref f) => {
            let mut states = BitSet::new();
            let mut nstates = k.states.clone();
            loop {
                e.map.insert(c.clone(), nstates.clone());
                nstates = try!(eval(&k, f, e));
                states = nstates.union(&try!(e.map.get(&(c.clone())).map(|r| (*r).clone()).ok_or(MuErrors::VarNotFound(c.clone())))).collect::<BitSet>();
                if states == nstates { 
                    break; 
                }
            }
            return Ok(states);
        }
    };
}