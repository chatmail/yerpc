//! # Derivation macros for yerpc

#![warn(missing_debug_implementations, missing_docs, clippy::wildcard_imports)]

extern crate darling;
use darling::ast::NestedMeta;
use darling::{FromAttributes, FromMeta};
#[cfg(feature = "openrpc")]
use openrpc::generate_openrpc_generator;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Item};

#[cfg(feature = "openrpc")]
mod openrpc;
mod parse;
mod rpc;
mod ts;
pub(crate) use parse::{Inputs, RpcInfo};
pub(crate) use rpc::generate_rpc_impl;
pub(crate) use ts::generate_typescript_generator;
pub(crate) mod util;

/// Generates the jsonrpc handler and types.
///
/// ### Root Attribute Arguments:
/// - `all_positional: bool` Positional mode means that the parameters of the RPC call are expected to be a JSON array,
///   which will be parsed as a tuple of this function's arguments.
/// - `ts_outdir: Option<String>` Set the path where typescript definitions are written to (relative to the crate root).
///   If not set, no typescript definitions will be written.
/// - `openrpc_outdir: Option<String>` Set the path where openrpc specification file will be written to (relative to the crate root).
///   If not set, no openrpc definition file will be written.
///
/// Note that you need to specify atleast one type definition output: `ts_outdir`, `openrpc_outdir` or both.
///
/// ### Method Attribute Arguments:
/// - `name: Option<String>` Set the name of the RPC method. Defaults to the function name.
/// - `notification: bool` Make this a notification method. Notifications are received like method calls but cannot
///   return anything.
/// - `positional: bool` Positional mode means that the parameters of the RPC call are expected to be a JSON array,
///   which will be parsed as a tuple of this function's arguments.
#[proc_macro_attribute]
pub fn rpc(attr: TokenStream, tokens: TokenStream) -> TokenStream {
    let item = parse_macro_input!(tokens as Item);
    match &item {
        Item::Impl(input) => {
            let attr_args = match NestedMeta::parse_meta_list(attr.into()) {
                Ok(v) => v,
                Err(err) => return TokenStream::from(darling::Error::from(err).write_errors()),
            };

            let attr_args = match RootAttrArgs::from_list(&attr_args) {
                Ok(args) => args,
                Err(err) => return TokenStream::from(err.write_errors()),
            };
            if attr_args.openrpc_outdir.is_none() && attr_args.ts_outdir.is_none() {
                return syn::Error::new_spanned(
                    item,
                    "The #[rpc] attribute needs atleast one type definition output. Please either set ts_outdir, openrpc_outdir or both.",
                )
                .to_compile_error().into()
            }

            let info = RpcInfo::from_impl(&attr_args, input);
            let ts_impl = if let Some(outdir) = attr_args.ts_outdir.as_ref() {
                generate_typescript_generator(&info,outdir)
            } else {
                quote!()
            };
            let rpc_impl = generate_rpc_impl(&info);

            #[cfg(feature = "openrpc")]
            let openrpc_impl = if let Some(outdir) = attr_args.openrpc_outdir.as_ref() {
                generate_openrpc_generator(&info, outdir)
            } else {
                quote!()
            };

            #[cfg(not(feature = "openrpc"))]
            let openrpc_impl = quote!();

            quote! {
                #item
                #rpc_impl
                #ts_impl
                #openrpc_impl
            }
        }
        Item::Fn(_) => quote!(#item),
        _ => syn::Error::new_spanned(
            item,
            "The #[rpc] attribute only works on impl and method items",
        )
        .to_compile_error(),
    }
    .into()
}

#[derive(FromMeta, Debug, Default)]
#[darling(default)]
pub(crate) struct RootAttrArgs {
    /// Positional mode means that the parameters of the RPC call are expected to be a JSON array,
    /// which will be parsed as a tuple of this function's arguments.
    all_positional: bool,
    /// Set the path where typescript definitions are written to (relative to the crate root).
    /// If not set, no typescript definitions will be written
    ts_outdir: Option<String>,
    /// Set the path where openrpc definitions will be written to (relative to the crate root).
    /// If not set, no openrpc definitions will be written.
    openrpc_outdir: Option<String>,
}

#[derive(FromAttributes, Debug, Default)]
#[darling(default, attributes(rpc))]
pub(crate) struct MethodAttrArgs {
    /// Set the name of the RPC method. Defaults to the function name.
    name: Option<String>,
    /// Make this a notification method. Notifications are received like method calls but cannot
    /// return anything.
    #[darling(default)]
    notification: bool,
    /// Positional mode means that the parameters of the RPC call are expected to be a JSON array,
    /// which will be parsed as a tuple of this function's arguments.
    positional: bool,
}
