use std::fmt;
use std::result;

use std::vec::Vec;

#[derive(Clone)]
pub struct AutHeader {
    pub first_state: u64,
    pub nr_of_transitions: u64,
    pub nr_of_states: u64
}

#[derive(Clone)]
pub struct AutEdge {
    pub start_state: u64,
    pub end_state: u64,
    pub label: String
}

#[derive(Clone)]
pub struct AutFile {
    pub header: AutHeader,
    pub edges: Vec<AutEdge>
}

impl fmt::Debug for AutFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AutHeader {{ first_state: {}, nr_of_transitions: {}, nr_of_states: {} }}", 
            self.header.first_state, 
            self.header.nr_of_transitions, 
            self.header.nr_of_states)
    }
}
peg! aut_grammar(r#"
use std::str::FromStr;
use std::vec::Vec;

use super::AutHeader;
use super::AutEdge;
use super::AutFile;

aut_header -> AutHeader
   = "des (" f:$([0-9]+) "," n1:$([0-9]+) "," n2:$([0-9]+) ")" " "* {
      AutHeader {
           first_state: u64::from_str(f).unwrap(),
           nr_of_transitions: u64::from_str(n1).unwrap(),
           nr_of_states: u64::from_str(n2).unwrap() }
   }

aut_edge -> AutEdge
   = "(" s:$([0-9]+) ",\"" lbl:$([a-zA-Z0-9,.() ]+) "\"," e:$([0-9]+) ")" {
      AutEdge {
          start_state: u64::from_str(s).unwrap(),
          label: lbl.to_owned(),
          end_state: u64::from_str(e).unwrap() }
   }

edges -> Vec<AutEdge>
  = e:aut_edge ** "\n" {
    e
  }

enter = "\r"? "\n"

#[pub]
aut_file -> AutFile
  = h:aut_header enter e:aut_edge ** enter enter? {
    AutFile {
        header: h,
        edges: e
  }
}
"#);

pub fn read_aut_file(s: &str) -> result::Result<AutFile, aut_grammar::ParseError> {
    return aut_grammar::aut_file(s);
}
