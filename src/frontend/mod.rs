pub mod tokenizer;
pub mod parser;

use parser::{Block, DataItem, DataVariant, Statement};
use std::{collections::HashMap, marker::PhantomData, ops::Add};

pub type IStr<'s> = (PhantomData<&'s ()>, Box<str>);

#[derive(Debug)]
pub enum Type<'s> {
	User {
		name: IStr<'s>,
		format: DataFormat<'s>
	},
	Integer
}

#[derive(Debug)]
pub enum GenericFormat<'s, V> {
	Marker,
	Unnamed {
		fields: HashMap<usize, usize>
	},
	Named {
		fields: HashMap<usize, StructField<'s>>,
		variants: V
	}
}

pub type DataFormat<'s> =
	GenericFormat<'s, HashMap<usize, EnumVariantFormat<'s>>>;

#[derive(Debug)]
pub struct EnumVariantFormat<'s> {
	name: IStr<'s>,
	format: GenericFormat<'s, ()>
}

#[derive(Debug)]
pub struct StructField<'s> {
	name: IStr<'s>,
	r#type: usize
}

#[derive(Debug)]
pub struct Function<'s> {
	name: IStr<'s>,
	code: Code<'s>
}

#[derive(Debug)]
pub struct Code<'s> {
	scope: Scope<'s>
}

#[derive(Debug)]
pub struct Scope<'s> {
	types: HashMap<usize, Type<'s>>,
	functions: HashMap<usize, Function<'s>>
}

trait Identifier: Copy {
	type Representation: Add<Output = Self::Representation> + Copy;

	const ONE: Self::Representation;
	const ZERO: Self::Representation;

	fn build(representation: Self::Representation) -> Self;
}

impl Identifier for usize {
	type Representation = usize;

	const ONE: usize = 1;
	const ZERO: usize = 0;

	fn build(representation: usize) -> Self {
		representation
	}
}

struct IDBuilder<T>(T::Representation)
	where T: Identifier;

impl<T> IDBuilder<T>
		where T: Identifier {
	fn new() -> Self {
		Self(T::ZERO)
	}

	fn next(&mut self) -> T {
		let id = T::build(self.0);
		self.0 = self.0 + T::ONE;
		id
	}
}

pub fn construct_main_representation(block: &Block) -> Code<'static> {
	// Identify types.
	let mut type_ids = IDBuilder::<usize>::new();
	let types: HashMap<_, _> = block.0.iter()
		.filter_map(Statement::data_item_ref)
		.map(|data| (data.name(), (type_ids.next(), data)))
		.collect();

	// Identify field types.
	let types = types.values()
		.map(|(id, data)| (
			*id,
			match data {
				DataItem::Single(variant) => {
					let (name, format) =
						construct_data_representation(variant, &types);
					Type::User {name, format}
				},

				DataItem::Multiple {name, variants} => {
					let mut variant_ids = IDBuilder::<usize>::new();
					let variants = variants.iter()
						.map(|variant| {
							let (name, format) =
								construct_data_representation(variant, &types);
							(variant_ids.next(), EnumVariantFormat {name, format})
						})
						.collect();

					Type::User {
						name: (PhantomData, name.clone()),
						format: DataFormat::Named {
							fields: HashMap::new(),
							variants
						}
					}
				}
			}
		))
		.collect();

	// Identify functions.
	let mut function_ids = IDBuilder::new();
	let functions = block.0.iter()
		.filter_map(Statement::function_item_ref)
		.map(|function| Function {
			name: (PhantomData, function.name.clone()),
			code: construct_main_representation(&function.body)
		})
		.map(|function| (function_ids.next(), function))
		.collect();

	Code {scope: Scope {types, functions}}
}

pub fn construct_data_representation<V>(variant: &DataVariant,
		types: &HashMap<&str, (usize, &DataItem)>)
			-> (IStr<'static>, GenericFormat<'static, V>) where V: Default {
	match variant {
		DataVariant::Marker {name} => (
			(PhantomData, name.clone()),
			GenericFormat::Marker
		),

		DataVariant::Tuple {name, fields} => {
			let mut field_ids = IDBuilder::<usize>::new();
			let fields = fields.iter()
				.map(|r#type| (field_ids.next(),
					types.get(&**r#type).expect("unknown type").0))
				.collect();

			(
				(PhantomData, name.clone()),
				GenericFormat::Unnamed {fields}
			)
		},

		DataVariant::Struct {name, fields} => {
			let mut field_ids = IDBuilder::<usize>::new();
			let fields = fields.iter()
				.map(|(name, r#type)| (
					field_ids.next(),
					StructField {
						name: (PhantomData, name.clone()),
						r#type: types.get(&**r#type).expect("unknown type").0
					}
				))
				.collect();

			(
				(PhantomData, name.clone()),
				GenericFormat::Named {fields, variants: Default::default()}
			)
		}
	}
}
