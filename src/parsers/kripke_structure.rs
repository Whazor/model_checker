use std::collections::HashSet;
use std::collections::HashMap;
use parsers::aldebaran::AutFile;
use std::hash::Hash;
use bit_set::BitSet;


#[derive(Clone)]
pub struct MixedKripkeStructure<L: Clone+Copy> {
    pub states: BitSet<u32>,
    pub init_states: BitSet<u32>,
    pub relations: HashMap<(u32, String), BitSet<u32>>,
    pub label: HashMap<u32, HashSet<L>>
}


pub fn from_aut_to_kripke(aut: &AutFile) -> MixedKripkeStructure<()> {
    let nr_of_states = aut.header.nr_of_states;
    let nr_of_transitions = aut.header.nr_of_transitions;
    let mut kripke = MixedKripkeStructure::<()> { 
        states: BitSet::<u32>::with_capacity(nr_of_states * 2),
        init_states: BitSet::<u32>::with_capacity(1),
        relations:HashMap::<(u32, String), BitSet<u32>>::with_capacity(nr_of_transitions * 2),
        label: HashMap::<u32, HashSet<()>>::with_capacity(nr_of_states * 2)
    };
    kripke.states.insert(aut.header.first_state as usize);
    kripke.init_states.insert(aut.header.first_state as usize);
    for edge in &aut.edges {
        kripke.states.insert(edge.start_state as usize);
        kripke.states.insert(edge.end_state as usize);
        let rel = kripke.relations.entry((edge.start_state as u32, edge.label.clone())).or_insert(BitSet::with_capacity(1));
        rel.insert(edge.end_state as usize);
    }
    return kripke;
}