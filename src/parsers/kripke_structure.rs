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
    let nr_of_transitions = aut.header.nr_of_transitions;
    let mut kripke = MixedKripkeStructure::<u64, ()> { 
        states: HashSet::<u64>::with_capacity(nr_of_states * 2),
        init_states: HashSet::<u64>::with_capacity(1),
        relations: Vec::<(u64, String, u64)>::with_capacity(nr_of_transitions * 2),
        label: HashMap::<u64, HashSet<()>>::with_capacity(nr_of_states * 2)
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