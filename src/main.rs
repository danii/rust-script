use std::iter::Peekable;

#[derive(Debug)]
enum Token {
	Identifier(Box<str>),
	KeywordFN,
	KeywordData,

	ParenLeft,
	ParenRight,
	BraceLeft,
	BraceRight,
	BracketLeft,
	BracketRight,
	ArrowLeft,
	ArrowRight,

	Period,
	Comma,
	Colon,
	SemiColon
}

struct Tokenizer<I>(Peekable<I>)
	where I: Iterator<Item = char>;

impl<I> Tokenizer<I>
		where I: Iterator<Item = char> {
	fn eat(&mut self) {
		self.0.next();
	}

	fn next_identifier(&mut self) -> Token {
		let mut name = String::new();
		while let Some('a'..='z' | 'A'..='Z') = self.0.peek()
			{name.push(self.0.next().unwrap())}

		let name = Box::<str>::from(name);
		match &*name {
			"fn" => Token::KeywordFN,
			"data" => Token::KeywordData,
			_ => Token::Identifier(name)
		}
	}

	fn skip_whitespace(&mut self) -> Option<Token> {
		while let Some(' ' | '\n' | '\t') = self.0.peek()
			{self.0.next();}

		self.next()
	}
}

impl<I> Iterator for Tokenizer<I>
		where I: Iterator<Item = char> {
	type Item = Token;

	fn next(&mut self) -> Option<Token> {
		Some(match self.0.peek()? {
			'a'..='z' | 'A'..='Z' => self.next_identifier(),
			' ' | '\n' | '\t' => self.skip_whitespace()?,

			'(' => {self.eat(); Token::ParenLeft},
			')' => {self.eat(); Token::ParenRight},
			'{' => {self.eat(); Token::BraceLeft},
			'}' => {self.eat(); Token::BraceRight},
			'[' => {self.eat(); Token::BracketLeft},
			']' => {self.eat(); Token::BracketRight},
			'<' => {self.eat(); Token::ArrowLeft},
			'>' => {self.eat(); Token::ArrowRight},

			'.' => {self.eat(); Token::Period},
			',' => {self.eat(); Token::Comma},
			':' => {self.eat(); Token::Colon},
			';' => {self.eat(); Token::SemiColon},
			_ => todo!()
		})
	}
}

const X: &str = include_str!("main.rsst");

fn main() {
	let t = Tokenizer(X.chars().peekable());

	for x in t {
		println!("{:?}", x);
	}
}