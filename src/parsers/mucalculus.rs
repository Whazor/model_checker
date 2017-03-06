use std::fmt;
use std::result;
use std::string;

#[derive(Clone)]
pub enum MuFormula {
    Action(String),
    Bool(bool),
    RecursionValue(String),
    And(Box<MuFormula>, Box<MuFormula>),
    Or(Box<MuFormula>, Box<MuFormula>),
    Not(Box<MuFormula>),
    DiamondOp (String, Box<MuFormula>),
    BoxOp (String, Box<MuFormula>),
    Mu(String, Box<MuFormula>),
    Nu(String, Box<MuFormula>),
}

impl string::ToString  for MuFormula {
    fn to_string(&self) -> String {
        let s = match self {
            &MuFormula::Action(ref a) => a.clone(),
            &MuFormula::Bool(b) => String::from(if b { "true" } else { "false" }),
            &MuFormula::RecursionValue(ref c) => c.clone(),
            &MuFormula::Not(ref f) => format!("!({})", f.to_string()),
            &MuFormula::And(ref f, ref g) => format!("({}&&{})", f.to_string(), g.to_string()),
            &MuFormula::Or(ref f, ref g) => format!("({}||{})", f.to_string(), g.to_string()),
            &MuFormula::DiamondOp (ref ac, ref f) => format!("<{}>{}",&ac,f.to_string()),
            &MuFormula::BoxOp (ref ac, ref f) => format!("[{}]{}",&ac,f.to_string()),
            &MuFormula::Mu(ref c, ref f) => format!("mu {}.{}",c,f.to_string()),
            &MuFormula::Nu(ref c, ref f) => format!("nu {}.{}",c,f.to_string()),
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

false -> MuFormula = "false" { MuFormula::Bool(false) };
true -> MuFormula = "true" { MuFormula::Bool(true) };

recursion_variable -> MuFormula = a:$([A-Z]) { MuFormula::RecursionValue(a.to_owned()) };

binaryAnd -> MuFormula = "(" f:formula "&&" g:formula ")" { MuFormula::And(box f, box g) };
binaryOr -> MuFormula = "(" f:formula "||" g:formula ")" { MuFormula::Or(box f, box g) };

action -> MuFormula = a:$([a-z_]+) { MuFormula::Action(a.to_owned()) };

diamond -> MuFormula = "<" a:$([a-z_]+) ">" f:formula { MuFormula::DiamondOp(a.to_owned(), box f) };
box -> MuFormula = "[" a:$([a-z_]+) "]" f:formula { MuFormula::BoxOp(a.to_owned(), box f) };

mu_point -> MuFormula = 'mu' c:$([A-Z]) '.' f:formula { MuFormula::Mu(c.to_owned(), box f) };
nu_point -> MuFormula = 'nu' c:$([A-Z]) '.' f:formula { MuFormula::Nu(c.to_owned(), box f) };

pub formula -> MuFormula = false / true / recursion_variable / binaryAnd / binaryOr / diamond / box / mu_point / nu_point;
"#);

pub fn read_mu_formula(s: &str) -> result::Result<MuFormula, mu_grammar::ParseError> {
    return mu_grammar::formula(s);
}
