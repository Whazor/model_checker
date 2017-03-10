use std::fmt;
use std::result;
use std::string;
use std::collections::HashSet;
use utils::collections::{merge_map, merge_set};
use std::hash::{Hash, Hasher, SipHasher};

#[derive(Clone)]
pub enum MuFormula {
    Action(usize, String),
    Bool(usize, bool),
    RecursionValue(usize, String),
    And(usize, Box<MuFormula>, Box<MuFormula>),
    Or(usize, Box<MuFormula>, Box<MuFormula>),
    Not(usize, Box<MuFormula>),
    DiamondOp (usize, String, Box<MuFormula>),
    BoxOp (usize, String, Box<MuFormula>),
    Mu(usize, String, Box<MuFormula>),
    Nu(usize, String, Box<MuFormula>),
}

fn position(mu: &MuFormula) -> usize {
    match *mu {
        MuFormula::Action(p, _) => p,
        MuFormula::Bool(p, _) => p,
        MuFormula::RecursionValue(p, _) => p,
        MuFormula::Not(p, _) => p,
        MuFormula::And(p, _, _) => p,
        MuFormula::Or(p, _, _) => p,
        MuFormula::DiamondOp (p, _, _) => p,
        MuFormula::BoxOp (p, _, _) => p,
        MuFormula::Mu(p, _, _) => p,
        MuFormula::Nu(p, _, _) => p,
    }
}

pub fn find_children(mu: &MuFormula) -> HashSet<MuFormula> {
    match *mu {
        // least fixpoint operator
        MuFormula::Mu(_, _, ref f) | MuFormula::Nu(_, _, ref f) | MuFormula::Not(_, ref f) | MuFormula::DiamondOp (_,  _, ref f) | MuFormula::BoxOp (_,  _, ref f)  => {
            let mut s = HashSet::new();
            s.insert(*f.clone());
            return merge_set(&find_children(f), &s);
        }
        MuFormula::And(_, ref f, ref g) | MuFormula::Or(_, ref f, ref g) => {
            let mut s = HashSet::new();
            s.insert(*f.clone());
            s.insert(*g.clone());
            return merge_set(&merge_set(&find_children(f), &find_children(g)), &s);
        }
        _ => {
            return HashSet::new();
        }
    }
}


impl Hash for MuFormula {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(position(self));
        state.finish();
    }
}

impl PartialEq for MuFormula {
    fn eq(&self, other: &MuFormula) -> bool {
        position(self) == position(other)
    }
}
impl Eq for MuFormula {}

impl string::ToString for MuFormula {
    fn to_string(&self) -> String {
        let s = match self {
            &MuFormula::Action(_, ref a) => a.clone(),
            &MuFormula::Bool(_, b) => String::from(if b { "true" } else { "false" }),
            &MuFormula::RecursionValue(_, ref c) => c.clone(),
            &MuFormula::Not(_, ref f) => format!("!({})", f.to_string()),
            &MuFormula::And(_, ref f, ref g) => format!("({}&&{})", f.to_string(), g.to_string()),
            &MuFormula::Or(_, ref f, ref g) => format!("({}||{})", f.to_string(), g.to_string()),
            &MuFormula::DiamondOp (_, ref ac, ref f) => format!("<{}>{}",&ac,f.to_string()),
            &MuFormula::BoxOp (_, ref ac, ref f) => format!("[{}]{}",&ac,f.to_string()),
            &MuFormula::Mu(_, ref c, ref f) => format!("mu {}.{}",c,f.to_string()),
            &MuFormula::Nu(_, ref c, ref f) => format!("nu {}.{}",c,f.to_string()),
        };
        s
    }
}

impl fmt::Debug for MuFormula {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

peg! mu_grammar(r#"
use parsers::mucalculus::MuFormula;

false -> MuFormula = p:#position "false" { MuFormula::Bool(p, false) };
true -> MuFormula = p:#position "true" { MuFormula::Bool(p, true) };

recursion_variable -> MuFormula = p:#position a:$([A-Z]) { MuFormula::RecursionValue(p, a.to_owned()) };

binaryAnd -> MuFormula = p:#position "(" f:formula "&&" g:formula ")" { MuFormula::And(p, box f, box g) };
binaryOr -> MuFormula = p:#position "(" f:formula "||" g:formula ")" { MuFormula::Or(p, box f, box g) };

action -> MuFormula = p:#position a:$([a-z_]+) { MuFormula::Action(p, a.to_owned()) };

diamond -> MuFormula = p:#position "<" a:$([a-z_]+) ">" f:formula { MuFormula::DiamondOp(p, a.to_owned(), box f) };
box -> MuFormula = p:#position "[" a:$([a-z_]+) "]" f:formula { MuFormula::BoxOp(p, a.to_owned(), box f) };

mu_point -> MuFormula = p:#position 'mu' c:$([A-Z]) '.' f:formula { MuFormula::Mu(p, c.to_owned(), box f) };
nu_point -> MuFormula = p:#position 'nu' c:$([A-Z]) '.' f:formula { MuFormula::Nu(p, c.to_owned(), box f) };

pub formula -> MuFormula = false / true / recursion_variable / binaryAnd / binaryOr / diamond / box / mu_point / nu_point;
"#);

pub fn read_mu_formula(s: &str) -> result::Result<MuFormula, mu_grammar::ParseError> {
    return mu_grammar::formula(s);
}
