use proc_macro2::TokenStream;
use quote::*;
use syn::{Attribute, Fields, ItemEnum, Lit, LitStr, Variant};

/// Find the tag attriubte and and remove it from the attrs list, returning the literal
fn take_tag_attr(attrs: &mut Vec<Attribute>) -> Option<Lit> {
	for i in 0..attrs.len() {
		let path = &attrs[i].path;
		let path = quote!(#path).to_string();
		if path == "tag" {
			let val = attrs[i].parse_args().unwrap();
			attrs.remove(i);
			return Some(val);
		}
	}

	None
}

/// Find the flat attriubte and the literal and remove it from the attrs list, returns if it was found
fn take_flat_attr(attrs: &mut Vec<Attribute>) -> bool {
	for i in 0..attrs.len() {
		let path = &attrs[i].path;
		let path = quote!(#path).to_string();
		if path == "flat" {
			attrs.remove(i);
			return true;
		}
	}

	false
}

/// Derives serde for a int tag (https://github.com/serde-rs/serde/issues/745 )
pub fn serialize_tag(mut input: ItemEnum, lit: &LitStr) -> TokenStream {
	let enum_ident = &input.ident;

	let mut deserialize_match = quote! {};
	let mut serialize_match = quote! {};

	for Variant { ident, fields, attrs, .. } in input.variants.iter_mut() {
		let mut fields_token = quote! {};
		let mut deserialize = quote! {};
		let tag_lit = take_tag_attr(attrs).expect("You should use `#[tag(num)]` on all fields!");

		let serialize = match fields {
			Fields::Named(named) => {
				let idents = named.named.iter().map(|field| field.ident.as_ref().unwrap());
				fields_token = quote! {{ #(#idents),* }};

				let mut serialize_entries = Vec::new();
				let mut serialize_stream = quote! {};
				let mut deserialize_entries = quote! {};
				let mut contains_flat = false;
				let mut length = 0;
				for field in &mut named.named {
					let ident = field.ident.as_ref().unwrap();
					let literal = LitStr::new(&ident.to_string(), ident.span());

					if take_flat_attr(&mut field.attrs) {
						contains_flat = true;
						serialize_stream = quote! {#serialize_stream serde::Serialize::serialize(&&#ident, FlatMapSerializer(&mut x)).unwrap();};
						deserialize_entries = quote! {#deserialize_entries #ident : serde_json::from_value(value).map_err(|e|Error::custom(e))?,};
					} else {
						length += 1;
						serialize_entries.push(quote! {(#literal, &#ident)});
						deserialize_entries = quote! {#deserialize_entries #ident : serde_json::from_value(value.get_mut(#literal).unwrap().take()).map_err(|e|Error::custom(e))?,};
					}
				}
				deserialize = quote! { {#deserialize_entries}};

				// We must use a map to serialize flat
				if contains_flat {
					quote! {
						let mut x = serializer.serialize_map(None)?;
						x.serialize_entry(#lit, &(#tag_lit as u64))?;
						#serialize_stream
						#(x.serialize_entry #serialize_entries ?;)*
						x.end()
					}
				} else {
					// Otherwise we use a struct
					let v = vec![quote!(1); length + 1];
					quote! {
						let mut x = serializer.serialize_struct("text", #(#v)+*)?;
						x.serialize_field(#lit, &(#tag_lit as u64))?;
						#(x.serialize_field #serialize_entries ?;)*
						x.end()
					}
				}
			}
			// Unnamed structs are flattened
			Fields::Unnamed(unnamed) => {
				let fields = unnamed.unnamed.iter().count();
				assert_eq!(fields, 1, "Macro only supports one tuple argument (it is flattened).");

				fields_token = quote! {(value)};

				deserialize = quote! { ( serde_json::from_value(value).unwrap() )};
				quote! {
					let mut x = serializer.serialize_map(None)?;
					x.serialize_entry(#lit, &(#tag_lit as u64))?;
					serde::Serialize::serialize(&&value, FlatMapSerializer(&mut x))?;
					x.end()
				}
			}
			// Unit structs just serialize type
			Fields::Unit => {
				quote! {
					let mut x = serializer.serialize_struct("Text", 1)?;
					x.serialize_field(#lit, &(#tag_lit as u64))?;
					x.end()
				}
			}
		};

		serialize_match = quote! {#serialize_match
			&#enum_ident :: #ident #fields_token => {
				#serialize
			},
		};

		deserialize_match = quote! {
			#deserialize_match
			#tag_lit => Ok(#enum_ident :: #ident #deserialize),
		}
	}

	// Create a with function for all of the fields of the struct

	serialize_match = quote! {


		impl Serialize for #enum_ident {
			fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
			where
				S: serde::Serializer,
			{
				match &self{
					#serialize_match
				}
			}
		}

		impl<'de> serde::Deserialize<'de> for #enum_ident {
			fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
				let mut value = serde_json::Value::deserialize(d)?;
				match value.get(#lit).and_then(serde_json::Value::as_u64).unwrap() {
					#deserialize_match
					type_ => panic!("unsupported type {:?}", type_),
				}
			}
		}
	};

	quote! { #input #serialize_match}
}
