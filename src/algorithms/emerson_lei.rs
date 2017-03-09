use parsers::mucalculus::MuFormula;
use parsers::kripke_structure::MixedKripkeStructure;
use std::collections::HashSet;
use std::collections::HashMap;
use std::hash::Hash;
use std::fmt::Debug;
use std::{thread, time};
use utils::collections::{merge_map, merge_set};

struct Environment<'time, S: Hash+Eq+Clone+Copy+'time+Debug> {
    map: &'time mut HashMap<String, HashSet<S>>
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
            GFP => match other { GFP => true, _ => false },
            LFP => match other { LFP => true, _ => false },
            &Bound::None => match *other { Bound::None => true, _ => false }
        }
    }
}
impl Eq for Bound {}


fn find_variables(mu: MuFormula, bound: Bound) -> HashMap<MuFormula, Bound> {
    let mut hm = HashMap::new();
    hm.insert(mu.clone(), bound.clone());
    match mu.clone() {
        // least fixpoint operator
        MuFormula::Mu(_, c, f) => {
            hm = merge_map(&find_variables(*f, Bound::Mu(mu)), &hm);
        }
        // greatest fixpoint operator
        MuFormula::Nu(_, c, f) => {
            hm = merge_map(&find_variables(*f, Bound::Nu(mu)), &hm);
        }
        MuFormula::Not(_, f) | MuFormula::DiamondOp (_,  _, f) | MuFormula::BoxOp (_,  _, f)  => {
            hm = merge_map(&find_variables(*f, bound), &hm);
        }
        MuFormula::And(_, f, g) | MuFormula::Or(_, f, g) => {
            hm = merge_map(&merge_map(&find_variables(*f, bound.clone()), &find_variables(*g, bound)), &hm);
        }
        _ => {}
    }
    return hm;
}

fn find_children(mu: MuFormula) -> HashSet<MuFormula> {
    match mu.clone() {
        // least fixpoint operator
        MuFormula::Mu(_, c, f) => {
            return find_children(*f);
        }
        // greatest fixpoint operator
        MuFormula::Nu(_, c, f) => {
            return find_children(*f);
        }
        MuFormula::Not(_, f) | MuFormula::DiamondOp (_,  _, f) | MuFormula::BoxOp (_,  _, f)  => {
            return find_children(*f);
        }
        MuFormula::And(_, f, g) | MuFormula::Or(_, f, g) => {
            return merge_set(&find_children(*f), &find_children(*g));
        }
        _ => {
            return HashSet::new();
        }
    }
}

fn free_variables(mu: MuFormula, vars: HashMap<MuFormula, Bound>) -> HashSet<String> {
    let mut hs = HashSet::new();
    for var in find_children(mu.clone()) {
        match var.clone() {
            MuFormula::RecursionValue(_, v) => {
                match vars.get(&var).unwrap_or(&Bound::None) {
                    &Bound::Nu(ref var_mu) | &Bound::Mu(ref var_mu) => {
                        if *var_mu != mu {
                            hs.insert(v);
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

pub fn evaluate<'time, S: Hash+Eq+Clone+Copy+'time+Debug, L: Clone+Copy>(k: &'time MixedKripkeStructure<S, L>, mu: MuFormula) -> Result<HashSet<S>, MuErrors> {
    let mut env = Environment {
        map: &mut HashMap::new()
     };
    let variables = find_variables(mu.clone(), Bound::None);

    for (key, bound) in variables.clone() {
        match key {
            MuFormula::RecursionValue(_, c) => { 
                match bound {
                    Bound::None => {},
                    LFP => {
                        env.map.insert(c, k.states.clone());
                    },
                    GFP => {
                        env.map.insert(c, HashSet::new());
                    }
                }
            }
            _ => {}
        }
    }

    return eval(variables, k, mu, &mut env);
}


fn eval<'time, S: Hash+Eq+Clone+Copy+'time+Debug, L: Clone+Copy>(
    vars: HashMap<MuFormula, Bound>,
    k: &'time MixedKripkeStructure<S, L>, 
    mu: MuFormula, 
    e: &'time mut Environment<S>
    ) -> Result<HashSet<S>, MuErrors> {
    
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
            let result = try!(eval(vars, &k, *f, e));
            return Ok(k.states.difference(&result).cloned().collect());
        },
        MuFormula::And(_, f, g) => {
            let left = try!(eval(vars.clone(),&k, *f, e));
            let right = try!(eval(vars,&k, *g, e));
            return Ok(left.intersection(&right).cloned().collect());
        },
        MuFormula::Or(_, f, g) => {
            let left = try!(eval(vars.clone(),&k, *f, e));
            let right = try!(eval(vars,&k, *g, e));
            return Ok(left.union(&right).cloned().collect());
        },

        // CTL
        MuFormula::Action(_, a) => { 
            //TODO: implement label function
            Ok(HashSet::new())
        },
        MuFormula::DiamondOp (_, ac, f) => { 
            let states = try!(eval(vars,&k, *f, e));
            let mut result = HashSet::new();
            for pat in &k.relations {
                if states.contains(&pat.2) && pat.1 == ac {
                    result.insert(pat.0);
                }
            }
            return Ok(result);
        },
        MuFormula::BoxOp (_, ac, f) => { 
            let states = try!(eval(vars,&k, *f, e));
            let mut result = HashSet::new();
            for pat in &k.relations {
                if states.contains(&pat.2) && pat.1 == ac {
                    result.insert(pat.0);
                }
            }
            return Ok(k.states.difference(&result).cloned().collect());
        },

        // mu calculus
        MuFormula::RecursionValue(_, c) => { 
            let r = e.map.get(&c).map(|r| (*r).clone()).ok_or(MuErrors::VarNotFound(c));
            // println!("rec: {:?}", r);
            return r;
        },

        // least fixpoint operator
        MuFormula::Mu(_, c, f) => {
            // search if bound is GFP
            match vars.clone().get(&mu).unwrap_or(&Bound::None) {
                &Bound::Nu(_) => {
                    // find child mu's
                    for child in find_children(mu) {
                        match child.clone() {
                            MuFormula::Mu(_, c2, _) => {
                                let has_free_variables = free_variables(child, vars.clone()).len() > 0;
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

            let mut states = HashSet::<S>::new();  //try!(e.map.get(&(c.clone())).map(|r| (*r).clone()).ok_or(MuErrors::VarNotFound(c.clone())));
            let mut cstates = states.clone();
            let mut nstates = HashSet::<S>::new();
            let mut iterations = 0;
            loop {
                states = try!(e.map.get(&(c.clone())).map(|r| (*r).clone()).ok_or( MuErrors::VarNotFound(c.clone()) ));
                nstates = try!(eval(vars.clone(), &k, *f.clone(), e));
                e.map.insert((c.clone()), nstates.clone());
                states = states.union(&nstates).cloned().collect();
                if cstates == nstates && states != nstates && iterations > 2  {
                    panic!("nu formula failed, states are the same! after {} iterations", iterations);
                } else {
                    cstates = nstates.clone();
                    iterations+=1;
                    if iterations > 10000 {
                        println!("too long");
                    }
                }
                if states == nstates { 
                    break; 
                }
            }

            return Ok(nstates);
        },

        // greatest fixpMut operator
        MuFormula::Nu(_, c, f) => {
            // search if bound is LFP
            match vars.clone().get(&mu).unwrap_or(&Bound::None) {
                &Bound::Mu(_) => {
                    // find child mu's
                    for child in find_children(mu) {
                        match child.clone() {
                            MuFormula::Nu(_, c2, _) => {
                                let has_free_variables = free_variables(child, vars.clone()).len() > 0;
                                if has_free_variables {
                                    e.map.insert(c2, HashSet::<S>::new());
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
            let mut states = HashSet::<S>::new();//try!(e.map.get(&(c.clone())).map(|r| (*r).clone()).ok_or(MuErrors::VarNotFound(c.clone())));
            let mut nstates = HashSet::<S>::new();
            let mut cstates = states.clone();
            let mut iterations = 0;
            loop {
                states = try!(e.map.get(&(c.clone())).map(|r| (*r).clone()).ok_or(MuErrors::VarNotFound(c.clone())));
                nstates = try!(eval(vars.clone(),&k, *f.clone(), e));
                e.map.insert((c.clone()), nstates.clone());
                states = states.intersection(&nstates).cloned().collect();
                if cstates == nstates && states != nstates && iterations > 2  {
                    panic!("nu formula failed, states are the same! after {} iterations", iterations);
                } else {
                    cstates = nstates.clone();
                    iterations+=1;
                    if iterations > 10000 {
                        println!("too long");
                    }
                }
                if states == nstates { 
                    break; 
                }
            }
            return Ok(states);
        },
    };
}