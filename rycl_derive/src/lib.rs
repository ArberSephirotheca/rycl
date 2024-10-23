extern crate proc_macro;
pub(crate) mod ty_check;

use proc_macro::TokenStream;
use quote::quote;
#[allow(unused_imports)]
use shared_type::{DeviceStructMarker, KernelFn, Primitive};
use smallvec::SmallVec;
use syn::{
    parse_macro_input, parse_quote, visit_mut::VisitMut, Error, Expr, ExprArray, ExprForLoop,
    Fields, FnArg, GenericParam, ItemFn, ItemStruct, Pat, PatIdent, PatType, TraitBound, Type,
    TypeParamBound,
};
use ty_check::*;

/// Marker trait for kernel functions, user should not implement this trait manually
/// This trait is used to check if the customize type is valid in kernel functions

// kernel attribute macro for GPU kernel functions
#[proc_macro_attribute]
pub fn kernel_fn(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(input as ItemFn);

    // Check if the function contains `num_thread_blocks: u32` and `thread_block_size: u32`
    let mut has_num_thread_blocks = false;
    let mut has_thread_block_size = false;
    let mut generic_params = GenericParamSet::new();
    let mut errors = SmallVec::<[proc_macro2::TokenStream; 4]>::new();

    // Extract the function's generic parameters (if any)
    // fixme: this does not work
    for param in input_fn.sig.generics.params.iter_mut() {
        if let GenericParam::Type(type_param) = param {
            // Add the DeviceStructMarker trait bound if it's missing
            if !type_param
                .bounds
                .iter()
                .any(|bound| matches!(bound, TypeParamBound::Trait(TraitBound { path, .. }) if path.is_ident("DeviceStructMarker")))
            {
                type_param.bounds.push(parse_quote!(DeviceStructMarker));
                generic_params.insert(type_param.ident.to_string());

            }
        }
    }
    for arg in &input_fn.sig.inputs {
        if let FnArg::Typed(PatType { pat, ty, .. }) = arg {
            if let Pat::Ident(PatIdent { ident, .. }) = &**pat {
                let arg_type = &**ty;
                if ident == "num_thread_blocks" && is_u32(arg_type) {
                    has_num_thread_blocks = true;
                }
                if ident == "thread_block_size" && is_u32(arg_type) {
                    has_thread_block_size = true;
                }
                if !is_valid_type(arg_type, &generic_params) {
                    errors.push(
                        Error::new_spanned(arg_type, format!("argument type is not allowed in kernel functions, allowed types are: {:?} and KernelStruct", ALLOWED_PRIMITIVE_TYPES))
                            .into_compile_error()
                    );
                }
            }
        }
    }

    if !has_num_thread_blocks || !has_thread_block_size {
        let mut missing_args = SmallVec::<[&str; 2]>::new();
        if !has_num_thread_blocks {
            missing_args.push("num_thread_blocks: u32");
        }
        if !has_thread_block_size {
            missing_args.push("thread_block_size: u32");
        }
        let error_msg = format!(
            "kernel function must contain arguments: {}",
            missing_args.join(" and ")
        );
        errors.push(Error::new_spanned(&input_fn.sig, error_msg).into_compile_error());
    }

    let expanded = quote! {
        impl KernelFn for #input_fn {}
    };
    let output = quote! {
        #(#errors)*
        #input_fn
    };

    print!("{}", output);

    output.into()
}

// kernel attribute macro for GPU kernel structs
#[proc_macro_attribute]
pub fn kernel_struct(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as ItemStruct);
    let struct_name = &input.ident;
    let mut generic_params = GenericParamSet::new();
    let mut errors = SmallVec::<[proc_macro2::TokenStream; 4]>::new();

    // Collect the generic parameters (if any)
    for param in input.generics.params.iter_mut() {
        if let GenericParam::Type(type_param) = param {
            // Add the Primitive trait bound if it's missing
            if !type_param
                .bounds
                .iter()
                .any(|bound| matches!(bound, TypeParamBound::Trait(TraitBound { path, .. }) if path.is_ident("DeviceStructMarker")))
            {
                type_param.bounds.push(parse_quote!(Primitive));
                generic_params.insert(type_param.ident.to_string());

            }
        }
    }

    if let Fields::Named(fields) = &input.fields {
        for field in fields.named.iter() {
            if !is_valid_type(&field.ty, &generic_params) {
                errors.push(
                    Error::new(
                        struct_name.span(),
                        format!(
                            "Field `{}` in struct `{}` is not a primitive type.",
                            field.ident.as_ref().unwrap(),
                            struct_name
                        ),
                    )
                    .to_compile_error(),
                );
            }
        }
    }

    let expanded = quote! {
        impl DeviceStructMarker for #struct_name {}
    };

    TokenStream::from(quote! {
        #(#errors)*
        #input
        #expanded
    })
}

struct KernelSyntaxValidator;

impl VisitMut for KernelSyntaxValidator {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        match expr {
            Expr::ForLoop(ExprForLoop {
                attrs,
                label,
                for_token,
                pat,
                in_token,
                expr,
                body,
            }) => {
                todo!()
            }

            Expr::Array(syn::ExprArray {
                attrs,
                bracket_token,
                elems,
            }) => {
                todo!()
            }
            Expr::Path(expr_path) => {
                todo!()
            }
            _ => {
                Error::new_spanned(expr, "expression not allowed in kernel functions")
                    .into_compile_error();
            }
        }
    }
}
