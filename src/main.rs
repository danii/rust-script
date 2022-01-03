use std::iter::Peekable;

#[derive(Debug, PartialEq, Eq)]
enum Token {
	Identifier(Box<str>),
	KeywordFn,
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
	/// Eats a character, disposing of it.
	#[allow(unused_must_use)] // Rationale: This is eat.
	fn eat(&mut self) {
		self.peeked_next();
	}

	/// Eats a character, and returns the provided value.
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
		while let Some('a'..='z' | 'A'..='Z') = self.peek()
			{name.push(self.next().unwrap())}

		let name = Box::<str>::from(name);
		match &*name {
			"fn" => Token::KeywordFn,
			"data" => Token::KeywordData,
			_ => Token::Identifier(name)
		}
	}
}

impl<I> Iterator for Tokenizer<I>
		where I: Iterator<Item = char> {
	type Item = Token;

	fn next(&mut self) -> Option<Token> {
		Some(match self.parse_whitespace()? {
			'a'..='z' | 'A'..='Z' => self.parse_identifier(),

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

			_ => todo!()
		})
	}
}

#[derive(Debug)]
struct Block(Vec<Statement>);

#[derive(Debug)]
enum Statement {
	DataItem(DataItem),
	FunctionItem(FunctionItem),
	Expression(Expression)
}

#[derive(Debug)]
enum Expression {}

#[derive(Debug)]
enum DataItem {
	Single(DataVariant),
	Multiple {
		name: Box<str>,
		variants: Vec<DataVariant>
	}
}

#[derive(Debug)]
enum DataVariant {
	Marker {
		name: Box<str>
	},
	Tuple {
		name: Box<str>,
		fields: Vec<Box<str>>
	},
	Struct {
		name: Box<str>,
		fields: Vec<(Box<str>, Box<str>)>
	}
}

#[derive(Debug)]
struct FunctionItem {
	name: Box<str>,
	arguments: Vec<(Box<str>, Box<str>)>,
	body: Block
}

struct Parser<I>(Peekable<I>)
	where I: Iterator<Item = Token>;

impl<I> Parser<I>
		where I: Iterator<Item = Token> {
	/// Eats a token, disposing of it.
	fn eat(&mut self) {
		match self.next() {
			Some(_) => (),
			None => unreachable!("called eat when there wasn't anything next")
		}
	}

	#[must_use = "all tokens should be consumed"]
	fn eat_identifier(&mut self) -> Box<str> {
		match self.next() {
			Some(Token::Identifier(name)) => name,
			Some(_) => unreachable!("called eat_identifier when an identifier wasn't next"),
			None => unreachable!("called eat_identifier when there wasn't anything next")
		}
	}

	/// Returns the next character, if any.
	fn next(&mut self) -> Option<Token> {
		self.0.next()
	}

	fn peek(&mut self) -> Option<&Token> {
		self.0.peek()
	}

	fn parse_block(&mut self) -> Block {
		let mut statements = Vec::new();

		loop {
			statements.push(match self.peek() {
				Some(Token::KeywordFn) =>
					Statement::FunctionItem(self.parse_function()),
				Some(Token::KeywordData) =>
					Statement::DataItem(self.parse_data()),
				_ => break Block(statements),
			})
		}
	}

	fn parse_function(&mut self) -> FunctionItem {
		assert_eq!(self.next(), Some(Token::KeywordFn));
		let name = self.eat_identifier(); // CHECKS WHERE?
		assert_eq!(self.next(), Some(Token::ParenLeft));
		assert_eq!(self.next(), Some(Token::ParenRight));

		assert_eq!(self.next(), Some(Token::BraceLeft));
		let body = self.parse_block();
		assert_eq!(self.next(), Some(Token::BraceRight));

		FunctionItem {name, arguments: Vec::new(), body}
	}

	fn parse_data(&mut self) -> DataItem {
		assert_eq!(self.next(), Some(Token::KeywordData));
		let name = self.eat_identifier();

		match self.next() {
			// Struct or Enum
			Some(Token::BraceLeft) => match self.next() {
				Some(Token::Identifier(variant)) => match self.peek() {
					// Definitely a Struct
					Some(Token::Colon) => {
						self.eat();
						let r#type = self.eat_identifier();
						let mut fields = vec![(variant, r#type)];

						loop {
							match self.next() {
								// Field
								Some(Token::Comma) => {
									let name = self.eat_identifier();
									assert_eq!(self.next(), Some(Token::Colon));
									let r#type = self.eat_identifier();

									fields.push((name, r#type))
								},

								// End
								Some(Token::BraceRight) =>
									break DataItem::Single(DataVariant::Struct {name, fields}),

								_ => unimplemented!()
							}
						}
					},

					// Definitely an Enum
					_ => { // TODO: Fix this whole branch, it's crazy.
						let variant = match self.next() {
							// Struct
							Some(Token::BraceLeft) => {
								let mut fields = Vec::new();
								loop {
									if let Some(Token::BraceRight) = self.peek() {
										self.eat();
										break DataVariant::Struct {name: variant, fields}
									}

									let name = self.eat_identifier();
									assert_eq!(self.next(), Some(Token::Colon));
									let r#type = self.eat_identifier();
									fields.push((name, r#type));

									match self.next() {
										Some(Token::Comma) => (),
										Some(Token::ParenRight) =>
											break DataVariant::Struct {name: variant, fields},
										_ => unimplemented!()
									}
								}
							},

							// Tuple
							Some(Token::ParenLeft) => {
								let mut fields = Vec::new();
								loop {
									if let Some(Token::ParenRight) = self.peek() {
										self.eat();
										break DataVariant::Tuple {name: variant, fields}
									}

									fields.push(self.eat_identifier());

									match self.next() {
										Some(Token::Comma) => (),
										Some(Token::ParenRight) =>
											break DataVariant::Tuple {name: variant, fields},
										_ => unimplemented!()
									}
								}
							},

							// Marker
							Some(Token::Colon) => DataVariant::Marker {name: variant},

							_ => unimplemented!()
						};
						let mut variants = vec![variant];

						match self.next() {
							Some(Token::Comma) => (),
							Some(Token::BraceRight) =>
								return DataItem::Multiple {name, variants}, // Ew!
							_ => unimplemented!()
						}
						loop {
							let variant = self.eat_identifier();
							variants.push(match self.next() {
								// Struct
								Some(Token::BraceLeft) => {
									let mut fields = Vec::new();
									loop {
										if let Some(Token::BraceRight) = self.peek() {
											self.eat();
											break DataVariant::Struct {name: variant, fields}
										}

										let name = self.eat_identifier();
										assert_eq!(self.next(), Some(Token::Colon));
										let r#type = self.eat_identifier();
										fields.push((name, r#type));

										match self.next() {
											Some(Token::Comma) => (),
											Some(Token::ParenRight) =>
												break DataVariant::Struct {name: variant, fields},
											_ => unimplemented!()
										}
									}
								},

								// Tuple
								Some(Token::ParenLeft) => {
									let mut fields = Vec::new();
									loop {
										if let Some(Token::ParenRight) = self.peek() {
											self.eat();
											break DataVariant::Tuple {name: variant, fields}
										}

										fields.push(self.eat_identifier());

										match self.next() {
											Some(Token::Comma) => (),
											Some(Token::ParenRight) =>
												break DataVariant::Tuple {name: variant, fields},
											_ => unimplemented!()
										}
									}
								},

								// Marker
								Some(Token::Colon) => DataVariant::Marker {name: variant},

								_ => unimplemented!()
							});

							match self.next() {
								Some(Token::Comma) => (),
								Some(Token::BraceRight) =>
									break DataItem::Multiple {name, variants},
								_ => unimplemented!()
							}
						}
					}
				},

				// Empty Enum
				// TODO: Should this be an empty struct?
				Some(Token::BraceRight) =>
					DataItem::Multiple {name, variants: Vec::new()},

				_ => unimplemented!()
			},

			// Tuple Struct
			Some(Token::ParenLeft) => {
				let mut fields = Vec::new();
				loop {
					if let Some(Token::ParenRight) = self.peek() {
						self.eat();
						break DataItem::Single(DataVariant::Tuple {name, fields})
					}

					fields.push(self.eat_identifier());

					match self.next() {
						Some(Token::Comma) => (),
						Some(Token::ParenRight) =>
							break DataItem::Single(DataVariant::Tuple {name, fields}),
						_ => unimplemented!()
					}
				}
			},

			// Marker Struct
			Some(Token::Colon) =>
				DataItem::Single(DataVariant::Marker {name}),

			_ => unimplemented!()
		}
	}
}

fn join<I, S>(iterator: I, join: impl AsRef<str>) -> String
		where I: Iterator<Item = S>, S: AsRef<str> {
	iterator.enumerate()
		.fold(String::new(), |mut joined, (index, value)| {
			if index != 0 {joined.push_str(join.as_ref())}
			joined.push_str(value.as_ref());
			joined
		})
}

fn transpile<I>(iterator: I) -> String
		where I: Iterator<Item = Statement> {
	let mut output = String::new();

	for statement in iterator {
		match statement {
			Statement::DataItem(DataItem::Single(variant)) => {
				let values;
				let name = match variant {
					DataVariant::Marker {name} => {
						values = Vec::new();
						name
					},
					DataVariant::Tuple {name, fields} => {
						values = fields.into_iter().enumerate().map(|(index, _)| format!("_{}", index).into()).collect();
						name
					},
					DataVariant::Struct {name, fields} => {
						values = fields.into_iter().map(|(name, _)| name).collect();
						name
					}
				};

				let arguments = join(values.iter(), ", ");
				let constructor = join(values.iter().map(|name| format!("this.{0} = {0};", name)), "\n\t\t");
				output.push_str(&format!(r#"class {} {{
	constructor({}) {{
		{}
	}}
}}"#, name, arguments, constructor));
			},
			_ => todo!()
		}
	}

	output
}

const X: &str = include_str!("main.rsst");

fn main() {
	let output = Parser(Tokenizer(X.chars().peekable()).peekable()).parse_block();
	let output = transpile(output.0.into_iter());
	println!("{}", output);
}
