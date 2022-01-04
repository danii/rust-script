use super::tokenizer::Token;
use std::iter::Peekable;

#[derive(Debug)]
pub struct Block(pub Vec<Statement>);

#[derive(Debug)]
pub enum Statement {
	DataItem(DataItem),
	FunctionItem(FunctionItem),
	Expression(Expression)
}

#[derive(Debug)]
pub enum Expression {}

#[derive(Clone, Debug)]
pub enum DataItem {
	Single(DataVariant),
	Multiple {
		name: Box<str>,
		variants: Vec<DataVariant>
	}
}

impl DataItem {
	pub fn name(&self) -> &str {
		match self {
			Self::Single(variant) => variant.name(),
			Self::Multiple {name, ..} => name
		}
	}
}

#[derive(Clone, Debug)]
pub enum DataVariant {
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

impl DataVariant {
	pub fn name(&self) -> &str {
		match self {
			Self::Marker {name} => &name,
			Self::Tuple {name, ..} => &name,
			Self::Struct {name, ..} => &name
		}
	}
}

#[derive(Debug)]
pub struct FunctionItem {
	pub name: Box<str>,
	pub arguments: Vec<(Box<str>, Box<str>)>,
	pub body: Block
}


pub struct Parser<I>(pub Peekable<I>)
	where I: Iterator<Item = Token>;

impl<I> Parser<I>
		where I: Iterator<Item = Token> {
	pub fn new(iterator: I) -> Self {
		Self(iterator.peekable())
	}
}

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

	pub fn parse_block(&mut self) -> Block {
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

	pub fn parse_function(&mut self) -> FunctionItem {
		assert_eq!(self.next(), Some(Token::KeywordFn));
		let name = self.eat_identifier(); // CHECKS WHERE?
		assert_eq!(self.next(), Some(Token::ParenLeft));
		assert_eq!(self.next(), Some(Token::ParenRight));

		assert_eq!(self.next(), Some(Token::BraceLeft));
		let body = self.parse_block();
		assert_eq!(self.next(), Some(Token::BraceRight));

		FunctionItem {name, arguments: Vec::new(), body}
	}

	pub fn parse_data(&mut self) -> DataItem {
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
										Some(Token::BraceRight) =>
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

								s => unimplemented!("{:?}", s)
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
						Some(Token::ParenRight) => {
							assert_eq!(self.next(), Some(Token::SemiColon));
							break DataItem::Single(DataVariant::Tuple {name, fields})
						},
						_ => unimplemented!()
					}
				}
			},

			// Marker Struct
			Some(Token::SemiColon) =>
				DataItem::Single(DataVariant::Marker {name}),

			_ => unimplemented!()
		}
	}
}
