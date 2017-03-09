#![feature(plugin)]
#![plugin(peg_syntax_ext)]
#![feature(box_syntax)]
mod algorithms;
#[macro_use]
mod parsers;

extern crate regex;
extern crate bufstream;
extern crate stopwatch;

mod utils;
