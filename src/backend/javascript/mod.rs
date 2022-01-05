use super::super::frontend::{Code, DataFormat, EnumVariantFormat};
use itertools::Itertools;
use std::{fmt::{Display, Formatter, Result as FMTResult}, iter::{empty, once}};

macro_rules! iter {
	() => {empty()};
	($first:expr $(, $($rest:expr),*)?) => {
		once($first).chain(iter![$($($rest),*)?])
	}
}

#[derive(Debug)]
pub struct Block(Vec<Statement>);

impl Display for Block {
	fn fmt(&self, f: &mut Formatter<'_>) -> FMTResult {
		(0..self.0.len())
			.try_for_each(|index| if index == 0
					|| !self.0[index - 1].requires_semicolon() {
				write!(f, "{}", self.0[index])
			} else {
				write!(f, ";{}", self.0[index])
			})
	}
}

#[derive(Debug)]
pub enum Statement {
	ClassItem(ClassItem),
	FunctionItem(),
	VarDeclaration(),
	LetDeclaration(),
	ConstDeclaration()
}

impl Statement {
	/// Whether or not this statement requires a `;` afterwards (ignoring ASI).
	pub fn requires_semicolon(&self) -> bool {
		match self {
			_ => false
		}
	}
}

impl Display for Statement {
	fn fmt(&self, f: &mut Formatter<'_>) -> FMTResult {
		match self {
			Self::ClassItem(class) => class.fmt(f),
			_ => todo!()
		}
	}
}

#[derive(Debug)]
pub struct ClassItem {
	name: Box<str>,
	fields: Vec<Box<str>>
}

impl Display for ClassItem {
	fn fmt(&self, f: &mut Formatter) -> FMTResult {
		write!(f, "class {}{{constructor(", self.name)?;
		(0..self.fields.len())
			.try_for_each(|index| if index == 0 {
				write!(f, "_{}", index)
			} else {
				write!(f, ",_{}", index)
			})?;
		write!(f, "){{")?;
		self.fields.iter().enumerate()
			.try_for_each(|(index, field)| if index == 0 {
				write!(f, "this[{:?}]=_{}", field, index)
			} else {
				write!(f, ";this[{:?}]=_{}", field, index)
			})?;
		write!(f, "}}}}")
	}
}

pub fn from_main_representation(code: &Code) -> Block {
	enum FormatFieldIterator<U, N, T>
			where U: Iterator<Item = T>, N: Iterator<Item = T> {
		Empty,
		Unnamed(U),
		Named(N)
	}

	impl<U, N, T> Iterator for FormatFieldIterator<U, N, T>
			where U: Iterator<Item = T>, N: Iterator<Item = T> {
		type Item = T;

		fn next(&mut self) -> Option<T> {
			match self {
				Self::Empty => None,
				Self::Unnamed(iter) => iter.next(),
				Self::Named(iter) => iter.next()
			}
		}
	}

	let classes = code.scope.types.iter()
		.filter_map(|(name, r#type)|
			r#type.format_ref().map(|r#type| (name, r#type)))
		.map(|(name, format)| match format {
			DataFormat::Marker => ClassItem {
				name: name.1.clone(),
				fields: Vec::new()
			},

			DataFormat::Unnamed {fields} => ClassItem {
				name: name.1.clone(),
				fields: (0..fields.len())
					.map(|index| format!("_{}", index).into_boxed_str())
					.collect()
			},

			DataFormat::Named {fields, variants} => ClassItem {
				name: name.1.clone(),
				fields: (!variants.is_empty())
					.then(|| "_variant".into())
					.into_iter()
					.chain(
						fields.keys()
							.map(|name| name.1.clone())
					)
					.chain(
						variants.values()
							.map(|variant| match variant {
								EnumVariantFormat::Marker => FormatFieldIterator::Empty,

								EnumVariantFormat::Unnamed {fields} =>
									FormatFieldIterator::Unnamed((0..fields.len())
										.map(|index| format!("_{}", index).into_boxed_str())),

								EnumVariantFormat::Named {fields, variants} =>
									FormatFieldIterator::Named(fields.keys()
										.map(|name| name.1.clone()))
							})
							.flatten()
							.dedup()
					)
					.collect()
			}
		})
		.map(|class| Statement::ClassItem(class));

	Block(classes.collect())
}
