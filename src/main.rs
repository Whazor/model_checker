#![feature(plugin)]
#![plugin(peg_syntax_ext)]
#![feature(box_syntax)]

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
mod parsers;
use parsers::aldebaran::read_aut_file;
use parsers::mucalculus::read_mu_formula;
use parsers::kripke_structure::from_aut_to_kripke;
mod algorithms;
use algorithms::naive;
use algorithms::emerson_lei;
use std::env;
mod utils;

extern crate rustyline;

use rustyline::error::ReadlineError;
use rustyline::completion::FilenameCompleter;
use rustyline::{Config, CompletionType, Editor};

extern crate stopwatch;
use stopwatch::{Stopwatch};

fn main() {    
    let mut aut = None;
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .build();
    let c = FilenameCompleter::new();
    let mut rl = Editor::with_config(config);
    rl.set_completer(Some(c));

    if let Err(_) = rl.load_history("history.txt") {
        println!("No previous history.");
    }
    println!("");
    println!("To open a file: open diner.lts");
    println!("To exit type: quit");
    println!("Furthermore, you can enter any µ-calculus formula");

    let mut args = false;
    let mut use_optimized = false;
    'outer: loop {
        let readline = if !args {
            args = true;
            let arr: Vec<String> = env::args().skip(1).collect();
            Ok(arr.join(" "))
            // Ok(env::args().skip(1).fold(env::args().nth(1), |a, x| format!("{} {}", a, x)))
        } else {
            rl.readline(">> ")
        };
        match readline {
            Ok(lines) => {
                let arr: Vec<&str> = lines.split(";").collect();
                for line in arr {
                    rl.add_history_entry(line.clone());
                    if line == "quit" || line == "exit" {
                        break 'outer;
                    }
                    if line == "switch" {
                        use_optimized = !use_optimized;
                        if use_optimized {
                            println!("Now using the Emerson Lei algorithm");
                        } else {
                            println!("Now using the naive algorithm");
                        }
                    } else if line.starts_with("open") {
                        let file_path_string = line.clone().replace("open ", "");
                        let path = Path::new(file_path_string.as_str());
                        let display = path.display();

                        match File::open(&path) {
                            Err(why) => { println!("couldn't open {}: {}", display, why.description()) },
                            Ok(mut file) => {
                                let mut s = String::new();
                                match file.read_to_string(&mut s) {
                                    Err(why) => println!("couldn't read {}: {}", display, why.description()),
                                    Ok(_) => {
                                        match read_aut_file(&s) {
                                            Ok(result) => { aut = Some(result); },
                                            Err(why) => println!("syntax error {}: {}", display, why)
                                        }
                                    }
                                }
                            }
                        };
                    } else {
                        match aut.clone() {
                            Some(aut_result) => {
                                let mu = read_mu_formula(line.replace(" ", "").as_str());
                                println!("States: {:?}", mu.clone());
                                match mu {
                                    Ok(mu) => {
                                        let result = if use_optimized {
                                            emerson_lei::evaluate(&from_aut_to_kripke(&aut_result), mu).unwrap()
                                        } else {
                                            naive::evaluate(&from_aut_to_kripke(&aut_result), mu).unwrap()
                                        };
                                        let n = result.clone().len() as u64;
                                        if n < 1000 {
                                            println!("{:?}", result);
                                        }
                                        println!("Number states from µ-formula: {}, total states: {}", n, aut_result.header.nr_of_states-1);
                                    },
                                    Err(why) => println!("couldn't parse mu: {}", why.description()),
                                }
                            },
                            None => { 
                                println!("No file loaded yet. Open file with: open diner.lts");
                            }
                        }
                    }
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break 'outer;
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break 'outer;
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break 'outer;
            }
        }
    }
    rl.save_history("history.txt").unwrap();
}
