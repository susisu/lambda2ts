mod compiler;

use std::io::{self, Read};

use crate::compiler::{generator, parser, transformer};

fn main() {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("failed to read");
    let program = parser::parse(&input).expect("parse error");
    let program = transformer::transform(&program);
    let code = generator::generate(&program);
    println!("{}", code);
}
