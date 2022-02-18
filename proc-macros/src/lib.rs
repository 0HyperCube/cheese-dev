extern crate proc_macro;

mod builder_pattern;
mod request_builder;
mod serialise_tag;

use proc_macro::TokenStream;
use syn::{parse::Nothing, parse_macro_input, ItemStruct, LitStr};

/// Generates the builder pattern and derives serialize for struct
#[proc_macro_attribute]
pub fn discord_struct(args: TokenStream, input: TokenStream) -> TokenStream {
	let _ = parse_macro_input!(args as Nothing);
	let mut input = parse_macro_input!(input as ItemStruct);

	let with_impl = builder_pattern::builder_pattern(&mut input);

	TokenStream::from(with_impl)
}

/// Builds a function to handle the request
#[proc_macro_attribute]
pub fn request(args: TokenStream, input: TokenStream) -> TokenStream {
	let arguments = parse_macro_input!(args as request_builder::RequestBuilderInput);
	let input = parse_macro_input!(input as syn::ItemStruct);

	let requests = arguments.process(input);
	TokenStream::from(requests)
}

/// Derives serde for a int tag (https://github.com/serde-rs/serde/issues/745 )
#[proc_macro_attribute]
pub fn serialise_tag(args: TokenStream, input: TokenStream) -> TokenStream {
	let arguments = parse_macro_input!(args as LitStr);
	let input = parse_macro_input!(input as syn::ItemEnum);

	TokenStream::from(serialise_tag::serialize_tag(input, &arguments))
}
