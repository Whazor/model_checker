use parsers::mucalculus::{MuFormula, find_children};
use parsers::kripke_structure::MixedKripkeStructure;
use std::collections::HashSet;
use std::collections::HashMap;
use std::hash::Hash;
use std::fmt::Debug;
use std::{thread, time};
use utils::collections::{merge_map, merge_set};
use bit_set::BitSet;

struct Environment<'time> {
    map: &'time mut HashMap<String, BitSet>
}

#[derive(Debug)]
pub enum MuErrors {
    VarNotFound(String)
}

#[derive(Clone)]
enum Bound { None, Mu(MuFormula), Nu(MuFormula) }

impl PartialEq for Bound {
    fn eq(&self, other: &Bound) -> bool {
        match self {
            &Bound::Nu(_) => match *other { Bound::Nu(_) => true, _ => false },
            &Bound::Mu(_) => match *other { Bound::Mu(_) => true, _ => false },
            &Bound::None => match *other { 
                Bound::None => true, _ => false
             }
        }
    }
}
impl Eq for Bound {}


fn find_variables(mu: &MuFormula, bound: Bound) -> HashMap<&MuFormula, Bound> {
    let mut hm = HashMap::new();
    hm.insert(mu, bound.clone());
    match *mu {
        // least fixpoint operator
        MuFormula::Mu(_, _, ref f) => {
            hm = merge_map(&find_variables(&f, Bound::Mu(mu.clone())), &hm);
        }
        // greatest fixpoint operator
        MuFormula::Nu(_, _, ref f) => {
            hm = merge_map(&find_variables(&f, Bound::Nu(mu.clone())), &hm);
        }
        MuFormula::Not(_, ref f) | MuFormula::DiamondOp (_,  _, ref f) | MuFormula::BoxOp (_,  _, ref f)  => {
            hm = merge_map(&find_variables(&f, bound), &hm);
        }
        MuFormula::And(_, ref f, ref g) | MuFormula::Or(_, ref f, ref g) => {
            hm = merge_map(&merge_map(&find_variables(&f, bound.clone()), &find_variables(&g, bound)), &hm);
        }
        _ => {}
    }
    return hm;
}

fn free_variables(mu: &MuFormula, vars: &HashMap<&MuFormula, Bound>) -> HashSet<String> {
    let mut hs = HashSet::new();
    for var in find_children(mu) {
        match var.clone() {
            MuFormula::RecursionValue(_, ref v) => {
                match vars.get(&var).unwrap_or(&Bound::None) {
                    &Bound::Nu(ref var_mu) | &Bound::Mu(ref var_mu) => {
                        if var_mu != mu {
                            hs.insert(v.clone());
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    return hs;
}

pub fn evaluate<'time, L: Clone+Copy>(k: &'time MixedKripkeStructure<L>, mu: MuFormula) -> Result<BitSet, MuErrors> {
    let mut env = Environment {
        map: &mut HashMap::new()
     };
    let variables = find_variables(&mu, Bound::None);

    for (key, bound) in variables.clone() {
        match *key {
            MuFormula::RecursionValue(_, ref c) => { 
                match bound {
                    Bound::None => {}
                    Bound::Mu(mu) => {
                        match mu {
                            MuFormula::Mu(_, ref c2, _) => {
                                if c == c2 {
                                    env.map.insert(c.clone(), BitSet::new());
                                }
                            }
                            _ => {}
                        }
      
                    }
                    Bound::Nu(mu) => {
                        match mu {
                            MuFormula::Nu(_, ref c2, _) => {
                                if c == c2 {
                                    env.map.insert(c.clone(), k.states.clone());
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    return eval(&variables, k, &mu, &mut env);
}


fn eval<'time, L: Clone+Copy>(
    vars: &HashMap<&MuFormula, Bound>,
    k: &'time MixedKripkeStructure<L>, 
    mu: &MuFormula, 
    e: &'time mut Environment
    ) -> Result<BitSet, MuErrors> {
    
    return match *mu {
        // logic
        MuFormula::Bool(_, ref b) => { 
            let hs = BitSet::new();
            if *b {
                return Ok(k.states.clone());
            }
            return Ok(hs);
        },
        MuFormula::Not(_, ref f) => {
            let result = try!(eval(vars, &k, f, e));
            return Ok(k.states.difference(&result).collect::<BitSet>());
        },
        MuFormula::And(_, ref f, ref g) => {
            let left = try!(eval(vars,&k, f, e));
            let right = try!(eval(vars,&k, g, e));
            return Ok(left.intersection(&right).collect::<BitSet>());
        },
        MuFormula::Or(_, ref f, ref g) => {
            let left = try!(eval(vars,&k, f, e));
            let right = try!(eval(vars,&k, g, e));
            return Ok(left.union(&right).collect::<BitSet>());
        },

        // CTL
        MuFormula::Action(_, _) => { 
            //TODO: implement label function
            Ok(BitSet::new())
        },
        MuFormula::DiamondOp (p, ref ac, ref f) => { 
            eval(vars, &k, 
                &MuFormula::Not(p, box MuFormula::BoxOp(p, ac.clone(), box MuFormula::Not(p, box *f.clone()))),
            e)
        },
        MuFormula::BoxOp (_, ref ac, ref f) => { 
            let states = try!(eval(vars, &k, f, e));
            let mut result = BitSet::new();
            for s in k.states.into_iter() {
                let mut insert = true;
                for pat in k.relations.get(&(s as u32, String::from(ac.clone()))).unwrap_or(&BitSet::new()) {
                    if !(states.contains(*&pat as usize)) {
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
            // search if bound is GFP
            match vars.get(mu).unwrap_or(&Bound::None) {
                &Bound::Nu(_) => {
                    // find child mu's
                    for child in find_children(mu) {
                        match child.clone() {
                            MuFormula::Mu(_, c2, _) => {
                                let has_free_variables = free_variables(&child, vars).len() > 0;
                                if has_free_variables {
                                    e.map.insert(c2, BitSet::new());
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }

            if !e.map.contains_key(&c.clone()) {
                e.map.insert(c.clone(), BitSet::new());
            }

            let mut states = BitSet::new();
            let mut nstates = BitSet::new();
            loop {
                states = try!(e.map.get(&(c.clone())).map(|r| (*r).clone()).ok_or(MuErrors::VarNotFound(c.clone())));
                nstates = try!(eval(&vars,&k, f, e));
                e.map.insert((c.clone()), nstates.clone());
                if states == nstates { 
                    break; 
                }
            }
            return Ok(states);
        },

        // greatest fixpMut operator
        MuFormula::Nu(_, ref c, ref f) => {
            // search if bound is LFP
            match vars.clone().get(&mu).unwrap_or(&Bound::None) {
                &Bound::Mu(_) => {
                    // find child mu's
                    for child in find_children(mu) {
                        match child.clone() {
                            MuFormula::Nu(_, c2, _) => {
                                let has_free_variables = free_variables(&child, vars).len() > 0;
                                if has_free_variables {
                                    e.map.insert(c2, k.states.clone());
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }

            if !e.map.contains_key(&c.clone()) {
                e.map.insert(c.clone(), k.states.clone());
            }
            let mut states = BitSet::new();
            let mut nstates = BitSet::new();
            loop {
                states = try!(e.map.get(&(c.clone())).map(|r| (*r).clone()).ok_or(MuErrors::VarNotFound(c.clone())));
                nstates = try!(eval(&vars,&k, f, e));
                e.map.insert((c.clone()), nstates.clone());
                if states == nstates { 
                    break; 
                }
            }
     
            return Ok(states);
        },
    };
}