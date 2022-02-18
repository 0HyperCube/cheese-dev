use proc_macro2::TokenStream;
use quote::*;
use syn::{Field, Fields, GenericArgument, ItemStruct, PathArguments, Type};

pub fn builder_pattern(input: &mut ItemStruct) -> TokenStream {
	let mut stream = quote! {};

	let fields = &input.fields;
	let mut fields_stream = quote! {};

	let ident = &input.ident;
	let generics = &input.generics;
	let attrs = &input.attrs;

	if let Fields::Named(fields) = &fields {
		// Create a with function for all of the fields of the struct
		for Field {
			ident,
			colon_token,
			ty,
			attrs: attributes,
			..
		} in &fields.named
		{
			// An optional extra attribute on the field (used for skip_serializing_if)
			let mut extra_attribute = quote! {};

			let mut added = false;
			let ident = ident.as_ref().unwrap();
			let argument = format_ident!("new_{}", ident);
			let concatenated = format_ident!("with_{}", ident);

			// Special handling for Vec and Option
			if let Type::Path(v) = ty {
				let vec_seg = v.path.segments.last().unwrap();

				if let PathArguments::AngleBracketed(args) = &vec_seg.arguments {
					if let Some(GenericArgument::Type(ty)) = args.args.last() {
						if vec_seg.ident.to_string() == "Vec" {
							added = true;

							stream = quote! {#stream
							/// Pushes to the Vector (builder pattern)
							pub fn #concatenated(mut self, #argument: impl Into<#ty>) -> Self{
								self.#ident.push(#argument.into());
								self
							}};
						} else if vec_seg.ident.to_string() == "Option" {
							// Without this, options serialize as `field: null` in json which discord rejects as bad request.
							extra_attribute = quote! {#[serde(skip_serializing_if = "Option::is_none", default)]};

							// Check if there is a vec nested inside this option (special behaviour of automatically generating a new vec if None)
							if let Type::Path(v) = ty {
								let vec_seg = v.path.segments.last().unwrap();
								if let PathArguments::AngleBracketed(args) = &v.path.segments.last().unwrap().arguments {
									if let Some(GenericArgument::Type(ty)) = args.args.last() {
										if vec_seg.ident.to_string() == "Vec" {
											added = true;
											stream = quote! {#stream
											/// Sets the field on the struct to Some(val) (builder pattern)
											pub fn #concatenated(mut self, #argument: impl Into<#ty>) -> Self{
												match &mut self.#ident{
													Some(#ident) => #ident.push(#argument.into()),
													None => self.#ident = Some(vec![#argument.into()])
												}
												self
											}}
										}
									}
								}
							}

							// If this is neither an option or a vec then add the builder in the normal way
							if !added {
								added = true;

								stream = quote! {#stream
								/// Sets the field on the struct to Some(val) (builder pattern)
								pub fn #concatenated(mut self, #ident: impl Into<#ty>) -> Self{
									self.#ident = Some(#ident.into());
									self
								}}
							};
						}
					}
				}
			}

			// Adds this field and the optional extra attribute back to the struct decleration
			fields_stream = quote! {#fields_stream #(#attributes)* #extra_attribute pub #ident #colon_token #ty,};

			// If we have not added a with function (i.e. it is not a vec or an option)
			if !added {
				stream = quote! {#stream
					/// Sets the field on the struct (builder pattern)
					pub fn #concatenated(mut self, #ident: impl Into<#ty>) -> Self{
						self.#ident= #ident.into();
						self
					}
				};
			}
		}
	}

	let decleartion = quote_spanned! {ident.span() => #ident #generics};

	stream = quote! {
		#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
		#(#attrs)*
		pub struct #decleartion{
			#fields_stream
		}
		impl #decleartion{
			/// Creates a new instance of this struct with default values
			pub fn new() -> Self {Self::default()}
			#stream
		}
	};

	stream
}
