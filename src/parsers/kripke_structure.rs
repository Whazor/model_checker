use std::collections::HashSet;
use std::collections::HashMap;
use parsers::aldebaran::AutFile;
use std::hash::Hash;

#[derive(Clone)]
pub struct MixedKripkeStructure<S: Hash+Eq+Clone+Copy, L: Clone+Copy> {
    pub states: HashSet<S>,
    pub init_states: HashSet<S>,
    pub relations: Vec<(S, String, S)>,
    pub label: HashMap<S, HashSet<L>>
}


pub fn from_aut_to_kripke(aut: &AutFile) -> MixedKripkeStructure<u64, ()> {
    let nr_of_states = aut.header.nr_of_states;
    let mut kripke = MixedKripkeStructure::<u64, ()> { 
        states: HashSet::<u64>::new(),
        init_states: HashSet::<u64>::new(),
        relations: Vec::<(u64, String, u64)>::new(),
        label: HashMap::<u64, HashSet<()>>::new()
    };
    kripke.states.insert(aut.header.first_state);
    kripke.init_states.insert(aut.header.first_state);
    for edge in &aut.edges {
        kripke.states.insert(edge.start_state);
        kripke.states.insert(edge.end_state);
        kripke.relations.push((edge.start_state, edge.label.clone(), edge.end_state));
    }
    return kripke;
}