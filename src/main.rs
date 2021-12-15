use std::iter::Peekable;

#[derive(Debug, PartialEq, Eq)]
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
	body: Vec<Statement>
}

#[derive(Debug)]
enum Statement {
	DataItem(DataItem),
	FunctionItem(FunctionItem),
	Expression(Expression)
}

#[derive(Debug)]
enum Expression {}

struct Parser<I>(Peekable<I>)
	where I: Iterator<Item = Token>;

impl<I> Parser<I>
		where I: Iterator<Item = Token> {
	fn next_function(&mut self) -> Option<FunctionItem> {
		assert_eq!(self.0.next(), Some(Token::KeywordFN));
		let name = if let Some(Token::Identifier(name)) = self.0.next() {name}
			else {return None};
		assert_eq!(self.0.next(), Some(Token::ParenLeft));
		assert_eq!(self.0.next(), Some(Token::ParenRight));

		assert_eq!(self.0.next(), Some(Token::BraceLeft));
		let mut body = Vec::new();
		while let Some(statement) = self.next() {body.push(statement)}
		assert_eq!(self.0.next(), Some(Token::BraceRight));

		Some(FunctionItem {
			name,
			arguments: Vec::new(),
			body
		})
	}

	fn next_data(&mut self) -> Option<DataItem> {
		assert_eq!(self.0.next(), Some(Token::KeywordData));
		let name = if let Some(Token::Identifier(name)) = self.0.next() {name}
			else {return None};
		assert_eq!(self.0.next(), Some(Token::BraceLeft));
		assert_eq!(self.0.next(), Some(Token::BraceRight));

		Some(DataItem::Single(DataVariant::Struct {name, fields: Vec::new()}))
	}
}

impl<I> Iterator for Parser<I>
		where I: Iterator<Item = Token> {
	type Item = Statement;

	fn next(&mut self) -> Option<Statement> {
		Some(match self.0.peek()? {
			Token::KeywordFN => Statement::FunctionItem(self.next_function()?),
			Token::KeywordData => Statement::DataItem(self.next_data()?),
			_ => return None
		})
	}
}

const X: &str = include_str!("main.rsst");

fn main() {
	let t = Parser(Tokenizer(X.chars().peekable()).peekable());

	4;

	for x in t {
		println!("{:?}", x);
	}
}