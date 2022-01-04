pub mod frontend;

use self::frontend::{parser::Parser, tokenizer::Tokenizer, construct_main_representation};
use std::{env::args, fs::read_to_string};

fn main() {
	let mut args = args();
	args.next();
	let input = args.next().unwrap();
	let output = args.next().unwrap();

	let input = read_to_string(input).unwrap();
	let block = Parser::new(Tokenizer::new(input.chars())).parse_block();
	construct_main_representation(&block);

	//let output = Parser(Tokenizer(X.chars().peekable()).peekable()).parse_block();
	//let output = transpile(output.0.into_iter());
	//println!("{}", output);
}
