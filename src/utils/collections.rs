use std::collections::HashSet;
use std::collections::HashMap;
use std::hash::Hash;

pub fn merge_map<K: Hash + Eq + Clone, V: Clone>(first_context: &HashMap<K, V>, second_context: &HashMap<K, V>) -> HashMap<K, V> {
    let mut new_context = HashMap::with_capacity(first_context.len() + second_context.len());
    for (key, value) in first_context.iter() {
        new_context.insert(key.clone(), value.clone());
    }
    for (key, value) in second_context.iter() {
        new_context.insert(key.clone(), value.clone());
    }
    new_context
}

pub fn merge_set<K: Hash + Eq + Clone>(first_context: &HashSet<K>, second_context: &HashSet<K>) -> HashSet<K> {
    let mut new_context = HashSet::with_capacity(first_context.len() + second_context.len());
    for key in first_context.iter() {
        new_context.insert(key.clone());
    }
    for key in second_context.iter() {
        new_context.insert(key.clone());
    }
    new_context
}
