use std::iter::Peekable;

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
	Identifier(Box<str>),

	KeywordFn,
	KeywordData,
	KeywordLet,

	LiteralNumber(Box<str>),
	LiteralTrue,
	LiteralFalse,

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
	SemiColon,

	Equals
}

pub struct Tokenizer<I>(pub Peekable<I>)
	where I: Iterator<Item = char>;

impl<I> Tokenizer<I>
		where I: Iterator<Item = char> {
	pub fn new(iterator: I) -> Self {
		Self(iterator.peekable())
	}
}

impl<I> Tokenizer<I>
		where I: Iterator<Item = char> {
	/// Eats a character, disposing of it.
	#[allow(unused_must_use)] // Rationale: This is eat.
	fn eat(&mut self) {
		self.peeked_next();
	}

	/// Eats a character, and returns the provided value.
	#[must_use = "if you do not need to return something, use eat"]
	fn eat_return<T>(&mut self, r#return: T) -> T {
		self.eat();
		r#return
	}

	/// Returns the next character, if any.
	#[must_use = "all characters should be consumed, if you already peeked this, you should use `eat`"]
	fn next(&mut self) -> Option<char> {
		self.0.next()
	}

	/// Returns the next character, assuming that the character was already
	/// peeked, and did infact, exist.
	#[must_use = "all characters should be consumed, you should use `eat`"]
	fn peeked_next(&mut self) -> char {
		match self.next() {
			Some(next) => next,
			None => unreachable!("called peeked_next when there wasn't anything next")
		}
	}

	/// Peeks the next character, if any.
	fn peek(&mut self) -> Option<char> {
		self.0.peek().map(Clone::clone)
	}

	/// Parses and discards all whitespace, and returns the last peeked non
	/// whitespace character.
	fn parse_whitespace(&mut self) -> Option<char> {
		loop {
			match self.peek()? {
				' ' | '\n' | '\r' | '\t' => self.eat(),
				character => break Some(character)
			}
		}
	}

	fn parse_identifier(&mut self) -> Token {
		let mut name = String::new();
		while let Some('a'..='z' | 'A'..='Z' | '_') = self.peek()
			{name.push(self.peeked_next())}

		let name = Box::<str>::from(name);
		match &*name {
			"fn" => Token::KeywordFn,
			"data" => Token::KeywordData,
			"let" => Token::KeywordLet,
			"true" => Token::LiteralTrue,
			"false" => Token::LiteralFalse,
			_ => Token::Identifier(name)
		}
	}

	fn parse_number(&mut self) -> Token {
		let mut number = String::new();
		while let Some('0'..='9') = self.peek()
			{number.push(self.peeked_next())}
		Token::LiteralNumber(Box::from(number))
	}
}

impl<I> Iterator for Tokenizer<I>
		where I: Iterator<Item = char> {
	type Item = Token;

	fn next(&mut self) -> Option<Token> {
		Some(match self.parse_whitespace()? {
			'a'..='z' | 'A'..='Z' | '_' => self.parse_identifier(),
			'0'..='9' => self.parse_number(),

			'(' => self.eat_return(Token::ParenLeft),
			')' => self.eat_return(Token::ParenRight),
			'{' => self.eat_return(Token::BraceLeft),
			'}' => self.eat_return(Token::BraceRight),
			'[' => self.eat_return(Token::BracketLeft),
			']' => self.eat_return(Token::BracketRight),
			'<' => self.eat_return(Token::ArrowLeft),
			'>' => self.eat_return(Token::ArrowRight),

			'.' => self.eat_return(Token::Period),
			',' => self.eat_return(Token::Comma),
			':' => self.eat_return(Token::Colon),
			';' => self.eat_return(Token::SemiColon),

			'=' => self.eat_return(Token::Equals),

			token => todo!("add failiure code; failed on token {:?}", token)
		})
	}
}
