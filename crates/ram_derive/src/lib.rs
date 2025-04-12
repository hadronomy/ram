use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields, Lit, Meta, Result, parse_macro_input};

#[proc_macro_derive(FromStaticText, attributes(static_text))]
pub fn derive_from_static_text(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match impl_from_static_text(&input) {
        Ok(tokens) => TokenStream::from(tokens),
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}

fn impl_from_static_text(input: &DeriveInput) -> Result<proc_macro2::TokenStream> {
    let enum_name = &input.ident;
    let variants = match &input.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "FromStaticText can only be derived for enums",
            ));
        }
    };

    let mut static_text_mappings = Vec::new();
    for variant in variants {
        // We only care about unit variants for static text mapping
        if !matches!(variant.fields, Fields::Unit) {
            continue;
        }
        let variant_ident = &variant.ident;
        if let Some(static_text_literal) = find_static_text_attribute(&variant.attrs)? {
            static_text_mappings.push((static_text_literal, variant_ident));
        }
    }

    // Generate match arms for modern const match
    let const_match_arms = static_text_mappings.iter().map(|(lit_str, variant_ident)| {
        let lit_str_value = lit_str.value();
        quote! { #lit_str_value => Some(Self::#variant_ident), }
    });

    let gen_modern = quote! {
         impl #enum_name {
            /// Attempts to convert a static string slice into an instance of this enum.
            ///
            /// This method checks if the input `text` matches any string defined
            /// in a `#[static_text("...")]` attribute on the enum's variants.
            #[inline]
            pub fn from_static_text(text: &str) -> Option<Self> {
                match text {
                    #( #const_match_arms )*
                    _ => None,
                }
            }
        }
    };

    Ok(gen_modern) // Use the simpler modern version
}

// Helper function to find static_text attribute and extract its value
fn find_static_text_attribute(attrs: &[Attribute]) -> Result<Option<syn::LitStr>> {
    for attr in attrs {
        if !attr.path().is_ident("static_text") {
            continue;
        }

        let meta = attr.meta.clone();
        match meta {
            Meta::List(meta_list) => {
                let tokens = meta_list.tokens.clone();
                let mut parser = tokens.into_iter().peekable();

                // Try to extract a string literal from the tokens
                if let Some(token) = parser.next() {
                    if let Ok(Lit::Str(lit_str)) =
                        syn::parse2::<Lit>(proc_macro2::TokenStream::from(token.clone()))
                    {
                        // Check that there are no more tokens
                        if parser.peek().is_none() {
                            return Ok(Some(lit_str));
                        }

                        return Err(syn::Error::new_spanned(
                            &meta_list,
                            "Expected exactly one string literal inside #[static_text(...)]",
                        ));
                    }
                }

                return Err(syn::Error::new_spanned(
                    &meta_list,
                    "Expected a string literal inside #[static_text(...)]",
                ));
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    &meta,
                    "Expected #[static_text(\"...\")], found different format",
                ));
            }
        }
    }
    Ok(None)
}
