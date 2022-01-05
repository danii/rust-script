pub mod tokenizer;
pub mod parser;

use parser::{Block, DataItem, DataVariant, Statement};
use std::{collections::{HashMap, HashSet}, marker::PhantomData};

pub type IStr<'s> = (PhantomData<&'s ()>, Box<str>);

#[derive(Debug)]
pub enum Type<'s> {
	User {
		format: DataFormat<'s>
	},
	Integer
}

impl<'s> Type<'s> {
	pub fn format_ref(&self) -> Option<&DataFormat<'s>> {
		match self {
			Self::User {format} => Some(format),
			_ => None
		}
	}
}

#[derive(Debug)]
pub enum GenericFormat<'s, V> {
	Marker,
	Unnamed {
		fields: Vec<IStr<'s>>
	},
	Named {
		fields: HashMap<IStr<'s>, IStr<'s>>,
		variants: V
	}
}

pub type DataFormat<'s> =
	GenericFormat<'s, HashMap<IStr<'s>, EnumVariantFormat<'s>>>;

pub type EnumVariantFormat<'s> =
	GenericFormat<'s, ()>;

#[derive(Debug)]
pub struct Function<'s> {
	code: Code<'s>
}

#[derive(Debug)]
pub struct Code<'s> {
	pub scope: Scope<'s>
}

#[derive(Debug, Default)]
pub struct Scope<'s> {
	pub types: HashMap<IStr<'s>, Type<'s>>,
	functions: HashMap<IStr<'s>, Function<'s>>
}

impl<'s> Scope<'s> {
	pub fn new() -> Self {
		Default::default()
	}
}

#[derive(Clone, Copy, Debug)]
// TODO: This type is outdated (and I wrote it like an hour ago lmao).
pub struct ScopeRef<'l, 'o, 's> {
	local: &'l Scope<'s>,
	outer: Option<&'o ScopeRef<'o, 'o, 's>>
}

impl<'l, 'o, 's> ScopeRef<'l, 'o, 's> {
	pub fn new(local: &'l Scope<'s>) -> Self {
		Self {local, outer: None}
	}

	pub fn r#in(&'o self, local: &'l Scope<'s>) -> Self {
		Self {local, outer: Some(self)}
	}

	pub fn has_type(&self, r#type: &IStr<'s>) -> bool {
		self.local.types.contains_key(r#type)
			|| self.outer.map(|scope| scope.has_type(r#type)).unwrap_or_default()
	}
}

pub fn construct_main_representation(block: &Block, scope: ScopeRef) -> Code<'static> {
	// Only used to verify that named types exist; types declared at the end of
	// the file may be used at the beginning of the same file.
	let type_names: HashSet<_> = block.0.iter()
		.filter_map(Statement::data_item_ref)
		.map(|data| data.name())
		.collect();

	// Process types.
	let types = block.0.iter()
		.filter_map(Statement::data_item_ref)
		.fold(HashMap::new(), |mut types, data| {
			let name = (PhantomData, data.name().into());
			let r#type = match data {
				DataItem::Single(variant) => {
					let (name, format) =
						construct_data_representation(variant, scope, &type_names);
					Type::User {format}
				},

				DataItem::Multiple {name, variants} => {
					let variants = variants.iter()
						.fold(HashMap::new(), |mut variants, variant| {
							let (name, format) =
								construct_data_representation(variant, scope, &type_names);

							// Variant Duplication Checks
							// TODO: Remove clone when IStr becomes an identifier.
							if variants.insert(name.clone(), format).is_some()
								{panic!("duplicate variant {:?}", name)}

							variants
						});

					Type::User {
						format: DataFormat::Named {
							fields: HashMap::new(),
							variants
						}
					}
				}
			};

			// Type Duplication Checks
			// TODO: Remove clone when IStr becomes an identifier.
			if types.insert(name.clone(), r#type).is_some()
				{panic!("duplicate type {:?}", name)}

			types
		});

	// Same deal as type_names.
	// TODO: How do we compile multiple files together???
	let function_names: HashSet<_> = block.0.iter()
		.filter_map(Statement::function_item_ref)
		.map(|function| &*function.name)
		.collect();

	// Process functions.
	let functions = block.0.iter()
		.filter_map(Statement::function_item_ref)
		.fold(HashMap::new(), |mut functions, function| {
			let name = (PhantomData, function.name.clone());
			// TODO: Fix scoping.
			let function = Function {
				code: construct_main_representation(&function.body, scope)
			};

			// Function Duplication Checks
			// TODO: Remove clone when IStr becomes an identifier.
			if functions.insert(name.clone(), function).is_some()
				{panic!("duplicate type {:?}", name)}

			functions
		});

	Code {scope: Scope {types, functions}}
}

pub fn construct_data_representation<V>(variant: &DataVariant,
		scope: ScopeRef, type_names: &HashSet<&str>)
			-> (IStr<'static>, GenericFormat<'static, V>) where V: Default {
	match variant {
		DataVariant::Marker {name} => (
			(PhantomData, name.clone()),
			GenericFormat::Marker
		),

		DataVariant::Tuple {name, fields} => {
			let fields: Vec<_> = fields.iter()
				.map(|r#type| {
					let r#type = (PhantomData, r#type.clone());

					// Type Reference Checks
					if !scope.has_type(&r#type) && !type_names.contains(&*r#type.1)
						{panic!("unknown type {:?}", r#type)}

					r#type
				})
				.collect();

			(
				(PhantomData, name.clone()),
				GenericFormat::Unnamed {fields}
			)
		},

		DataVariant::Struct {name, fields} => {
			let fields = fields.iter()
				.fold(HashMap::new(), |mut fields, (name, r#type)| {
					let r#type = (PhantomData, r#type.clone());
					let name = (PhantomData, name.clone());

					// Type Reference & Field Duplication Checks
					if !scope.has_type(&r#type) && !type_names.contains(&*r#type.1)
						{panic!("unknown type {:?}", r#type)}
					// TODO: Remove clone when IStr becomes an identifier.
					if fields.insert(name.clone(), r#type).is_some()
						{panic!("duplicate field {:?}", name)}

					fields
				});

			(
				(PhantomData, name.clone()),
				GenericFormat::Named {fields, variants: Default::default()}
			)
		}
	}
}
