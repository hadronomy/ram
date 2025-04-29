//! Procedural macros for hir_analysis
//!
//! This crate provides procedural macros for defining analysis passes and their dependencies.

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Ident, LitStr, Token, Type};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

/// Attribute macro for defining an analysis pass
///
/// This macro generates the implementation of the `AnalysisPass` trait for a struct.
/// It takes an optional `output` parameter that specifies the output type of the pass.
///
/// # Examples
///
/// ```
/// #[analysis_pass(output = "ControlFlowGraph")]
/// struct ControlFlowAnalysisPass;
/// ```
#[proc_macro_attribute]
pub fn analysis_pass(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the attribute arguments
    let attr_args = parse_macro_input!(attr as AnalysisPassArgs);
    
    // Parse the item
    let input = parse_macro_input!(item as DeriveInput);
    
    // Get the name of the struct
    let name = &input.ident;
    
    // Get the output type
    let output_type = attr_args.output.unwrap_or_else(|| {
        // If no output type is specified, use a default
        syn::parse_str::<Type>("()").unwrap()
    });
    
    // Generate the implementation
    let expanded = quote! {
        #input
        
        impl hir_analysis::analysis::AnalysisPass for #name {
            type Output = #output_type;
            
            fn run<'db, 'body>(
                &self,
                ctx: &mut hir_analysis::AnalysisContext<'db, 'body>,
                dependencies: &hir_analysis::analysis::results::AnalysisResultsCache,
            ) -> Self::Output {
                // Call the implementation method
                self.run_impl(ctx, dependencies)
            }
        }
    };
    
    expanded.into()
}

/// Attribute macro for defining dependencies of an analysis pass
///
/// This macro generates the implementation of the `dependencies` method for an analysis pass.
/// It takes a list of pass types that this pass depends on.
///
/// # Examples
///
/// ```
/// #[depends_on(ControlFlowAnalysisPass, DataFlowAnalysisPass)]
/// struct OptimizationAnalysisPass;
/// ```
#[proc_macro_attribute]
pub fn depends_on(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the attribute arguments
    let attr_args = parse_macro_input!(attr as DependsOnArgs);
    
    // Parse the item
    let input = parse_macro_input!(item as DeriveInput);
    
    // Get the name of the struct
    let name = &input.ident;
    
    // Get the dependencies
    let dependencies = &attr_args.dependencies;
    
    // Generate the implementation
    let expanded = quote! {
        #input
        
        impl #name {
            fn dependencies(&self) -> Vec<std::any::TypeId> {
                vec![
                    #(std::any::TypeId::of::<#dependencies>()),*
                ]
            }
        }
    };
    
    expanded.into()
}

/// Attribute macro for defining an analysis pass with dependencies
///
/// This macro combines the functionality of `analysis_pass` and `depends_on`.
/// It takes an `output` parameter and a `deps` parameter.
///
/// # Examples
///
/// ```
/// #[analysis_pass_with_deps(output = "OptimizationResults", deps = [ControlFlowAnalysisPass, DataFlowAnalysisPass])]
/// struct OptimizationAnalysisPass;
/// ```
#[proc_macro_attribute]
pub fn analysis_pass_with_deps(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the attribute arguments
    let attr_args = parse_macro_input!(attr as AnalysisPassWithDepsArgs);
    
    // Parse the item
    let input = parse_macro_input!(item as DeriveInput);
    
    // Get the name of the struct
    let name = &input.ident;
    
    // Get the output type
    let output_type = attr_args.output.unwrap_or_else(|| {
        // If no output type is specified, use a default
        syn::parse_str::<Type>("()").unwrap()
    });
    
    // Get the dependencies
    let dependencies = &attr_args.dependencies;
    
    // Generate the implementation
    let expanded = quote! {
        #input
        
        impl hir_analysis::analysis::AnalysisPass for #name {
            type Output = #output_type;
            
            fn run<'db, 'body>(
                &self,
                ctx: &mut hir_analysis::AnalysisContext<'db, 'body>,
                dependencies: &hir_analysis::analysis::results::AnalysisResultsCache,
            ) -> Self::Output {
                // Call the implementation method
                self.run_impl(ctx, dependencies)
            }
            
            fn dependencies(&self) -> Vec<std::any::TypeId> {
                vec![
                    #(std::any::TypeId::of::<#dependencies>()),*
                ]
            }
        }
    };
    
    expanded.into()
}

/// Arguments for the `analysis_pass` attribute
struct AnalysisPassArgs {
    output: Option<Type>,
}

impl Parse for AnalysisPassArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // If the input is empty, return default values
        if input.is_empty() {
            return Ok(Self { output: None });
        }
        
        // Parse the output type
        let mut output = None;
        
        // Parse key-value pairs
        let content;
        syn::parenthesized!(content in input);
        let pairs = Punctuated::<KeyValue, Token![,]>::parse_terminated(&content)?;
        
        for pair in pairs {
            if pair.key == "output" {
                output = Some(syn::parse_str(&pair.value)?);
            } else {
                return Err(syn::Error::new_spanned(
                    pair.key.to_token_stream(),
                    format!("Unknown parameter: {}", pair.key),
                ));
            }
        }
        
        Ok(Self { output })
    }
}

/// Arguments for the `depends_on` attribute
struct DependsOnArgs {
    dependencies: Punctuated<Type, Token![,]>,
}

impl Parse for DependsOnArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse the dependencies
        let dependencies = Punctuated::parse_terminated(input)?;
        
        Ok(Self { dependencies })
    }
}

/// Arguments for the `analysis_pass_with_deps` attribute
struct AnalysisPassWithDepsArgs {
    output: Option<Type>,
    dependencies: Punctuated<Type, Token![,]>,
}

impl Parse for AnalysisPassWithDepsArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // If the input is empty, return default values
        if input.is_empty() {
            return Ok(Self {
                output: None,
                dependencies: Punctuated::new(),
            });
        }
        
        // Parse the output type and dependencies
        let mut output = None;
        let mut dependencies = Punctuated::new();
        
        // Parse key-value pairs
        let content;
        syn::parenthesized!(content in input);
        let pairs = Punctuated::<KeyValue, Token![,]>::parse_terminated(&content)?;
        
        for pair in pairs {
            if pair.key == "output" {
                output = Some(syn::parse_str(&pair.value)?);
            } else if pair.key == "deps" {
                // Parse the dependencies list
                let deps_str = pair.value.trim_start_matches('[').trim_end_matches(']');
                let deps = deps_str.split(',').map(|s| s.trim());
                
                for dep in deps {
                    if !dep.is_empty() {
                        dependencies.push(syn::parse_str(dep)?);
                    }
                }
            } else {
                return Err(syn::Error::new_spanned(
                    pair.key.to_token_stream(),
                    format!("Unknown parameter: {}", pair.key),
                ));
            }
        }
        
        Ok(Self {
            output,
            dependencies,
        })
    }
}

/// A key-value pair for attribute arguments
struct KeyValue {
    key: Ident,
    value: String,
}

impl Parse for KeyValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse the key
        let key = input.parse()?;
        
        // Parse the equals sign
        input.parse::<Token![=]>()?;
        
        // Parse the value
        let value = if input.peek(LitStr) {
            let lit: LitStr = input.parse()?;
            lit.value()
        } else {
            let tokens = input.parse::<proc_macro2::TokenStream>()?;
            tokens.to_string()
        };
        
        Ok(Self { key, value })
    }
}
