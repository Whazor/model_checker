use std::vec::Vec;
use std::str;
use std::str::FromStr;
use regex::Regex;

use bufstream::BufStream;
use std::io::BufRead;
use std::fs::File;

use stopwatch::{Stopwatch};


#[derive(Debug, Clone, PartialEq)]
pub struct AutHeader {
    pub first_state: u64,
    pub nr_of_transitions: usize,
    pub nr_of_states: usize
}

#[derive(Debug, Clone, PartialEq)]
pub struct AutEdge {
    pub start_state: u64,
    pub end_state: u64,
    pub label: String
}

#[derive(Debug, Clone, PartialEq)]
pub struct AutFile {
    pub header: AutHeader,
    pub edges: Vec<AutEdge>
}

// impl fmt::Debug for AutFile {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "AutHeader {{ first_state: {}, nr_of_transitions: {}, nr_of_states: {} }}", 
//             self.header.first_state, 
//             self.header.nr_of_transitions, 
//             self.header.nr_of_states)
//     }
// }


// #[derive(Debug)]
// pub enum AutFileError {
//     VarNotFound(String)
// }


pub fn read_aut_file(file_stream: File) -> AutFile {
    let bf = BufStream::new(file_stream);

    let mut header = None;
    let mut edges = vec!();

    let sw = Stopwatch::start_new();
    let mut iterations: u32 = 0;
    let mut lines = bf.lines();

    let line = lines.next().unwrap();
    let mut s = line.unwrap();
    let re = Regex::new(r"^des \(([0-9]+),([0-9]+),([0-9]+)\)").unwrap();
    let cap = re.captures(s.as_str()).unwrap();
    header = Some(AutHeader { 
        first_state: u64::from_str(&cap[1]).unwrap(), 
        nr_of_transitions: usize::from_str(&cap[2]).unwrap(), 
        nr_of_states: usize::from_str(&cap[3]).unwrap() });

    for line in lines {
        let mut s = line.unwrap();
        if s.len() > 1 {
            s.remove(0);
            s.pop();
            let mut split = s.split(",");
            let start = u64::from_str(split.next().unwrap().clone()).unwrap();
            let mut label = String::from(split.next().unwrap().clone());
            let end = u64::from_str(split.next().unwrap().clone()).unwrap();
            label.remove(0);
            label.pop();
            edges.push(AutEdge { 
                start_state: start, 
                label: label, 
                end_state: end
            });
        }
    }
    return AutFile { header: header.unwrap(), edges: edges };
}


// pub fn read_aut_file(s: &str) -> IResult<&[u8], AutFile> {
//     return parse_aut(b"fsdf");
// }
// peg! aut_grammar(r#"
// use std::str::FromStr;
// use std::vec::Vec;

// use super::AutHeader;
// use super::AutEdge;
// use super::AutFile;

// aut_header -> AutHeader
//    = "des (" f:$([0-9]+) "," n1:$([0-9]+) "," n2:$([0-9]+) ")" " "* {
//       AutHeader {
//            first_state: u64::from_str(f).unwrap(),
//            nr_of_transitions: u64::from_str(n1).unwrap(),
//            nr_of_states: u64::from_str(n2).unwrap() }
//    }

// aut_edge -> AutEdge
//    = "(" s:$([0-9]+) ",\"" lbl:$([a-zA-Z0-9,.() ]+) "\"," e:$([0-9]+) ")" {
//       AutEdge {
//           start_state: u64::from_str(s).unwrap(),
//           label: lbl.to_owned(),
//           end_state: u64::from_str(e).unwrap() }
//    }

// edges -> Vec<AutEdge>
//   = e:aut_edge ** "\n" {
//     e
//   }

// enter = "\r"? "\n"

// #[pub]
// aut_file -> AutFile
//   = h:aut_header enter e:aut_edge ** enter enter? {
//     AutFile {
//         header: h,
//         edges: e
//   }
// }
// "#);

// pub fn read_aut_file(s: &str) -> result::Result<AutFile, aut_grammar::ParseError> {
//     println!("");
//     return aut_grammar::aut_file(s);
// }
