use crate::{util::extract_result_ty, Inputs, RpcInfo};
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
pub(crate) fn generate_bindings_impl(info: &RpcInfo) -> TokenStream {
    let mut gen_types = vec![];
    let mut gen_methods_ts = vec![];
    let mut gen_methods_qt = vec![];
    for method in &info.methods {
        let (is_positional, gen_args) = match &method.input {
            Inputs::Positional(ref inputs) => {
                let mut gen_args = vec![];
                for (i, input) in inputs.iter().enumerate() {
                    let ty = input.ty;
                    let name = input
                        .ident
                        .map_or_else(|| format!("arg{}", i + 1), ToString::to_string)
                        .to_case(Case::Camel);
                    gen_types.push(quote!(#ty));
                    gen_args.push(quote!((#name.to_string(), &<#ty as TypeDef>::INFO)))
                }
                (true, gen_args)
            }
            Inputs::Structured(Some(input)) => {
                let mut gen_args = vec![];
                let ty = input.ty;
                let name = input
                    .ident
                    .map_or_else(|| "params".to_string(), ToString::to_string)
                    .to_case(Case::Camel);
                gen_types.push(quote!(#ty));
                gen_args.push(quote!((#name.to_string(), &<#ty as TypeDef>::INFO)));
                (false, gen_args)
            }
            Inputs::Structured(None) => (false, vec![]),
        };
        let gen_output = match (method.output, method.is_notification) {
            (_, true) | (None, _) => quote!(None),
            (Some(ty), false) => {
                let ty = extract_result_ty(ty);
                gen_types.push(quote!(#ty));
                quote!(Some(&<#ty as TypeDef>::INFO))
            }
        };
        let rpc_name_camel = method.name.to_case(Case::Camel);
        let rpc_name = &method.name;
        let is_notification = method.is_notification;
        let docs = if let Some(docs) = &method.docs {
            quote!(Some(#docs))
        } else {
            quote!(None)
        };
        gen_methods_ts.push(quote!(
                let args = vec![#(#gen_args),*];
                let method = Method::new(#rpc_name_camel, #rpc_name, args, #gen_output, #is_notification, #is_positional, #docs);
                out.push_str(&method.to_string_ts(root_namespace));
        ));
        gen_methods_qt.push(quote!(
                let args = vec![#(#gen_args),*];
                let method = Method::new(#rpc_name_camel, #rpc_name, args, #gen_output, #is_notification, #is_positional, #docs);
                out.push_str(&method.to_string_qt());
        ));
    }

    let mut all_types: Vec<String> = gen_types
        .clone()
        .into_iter()
        .map(|ts| ts.to_string())
        .collect();
    all_types.sort();
    all_types.dedup();
    let all_types: Vec<TokenStream> = all_types.into_iter().map(|s| s.parse().unwrap()).collect();

    let ts = ts_impl(&all_types, &gen_methods_ts);
    let qt = qt_impl(&all_types, &gen_methods_qt);
    quote! {
        #ts
        #qt
    }
}

fn ts_impl(all_types: &[TokenStream], gen_methods: &[TokenStream]) -> TokenStream {
    let ts_base = include_str!("client.ts");

    quote! {
        /// Write typescript bindings for the JSON-RPC API.
        pub fn write_ts_bindings(outdir: &::std::path::Path) {
            use ::yerpc::typescript::type_def::{TypeDef, type_expr::TypeInfo, DefinitionFileOptions};
            use ::yerpc::{method::Method, typescript::{typedef_to_expr_string, export_types_to_file}};
            use ::std::{fs, path::Path};
            use ::std::io::Write;

            fs::create_dir_all(&outdir).expect(&format!("Failed to create directory `{}`", outdir.display()));

            // Create helper type with all exported types.
            // #(#gen_definitions)*
            #[derive(TypeDef)]
            struct __AllTyps(#(#all_types),*);
            // Write typescript types to file.
            export_types_to_file::<__AllTyps>(&outdir.join("types.ts"), None).expect("Failed to write TS out");
            // remove __AllTyps ts type from output,
            // it's only used as a woraround to export all types and is not needed anymore now
            let new_content = {
                let string =
                    ::std::fs::read_to_string(&outdir.join("types.ts")).expect("Failed to find TS out");
                if let Some(index) = string.find("export type __AllTyps") {
                    string[..index].to_string()
                } else {
                    panic!("did not find __AllTyps in TS out");
                }
            };
            ::std::fs::File::create(&outdir.join("types.ts"))
                .expect("failed to open TS out")
                .write_all(new_content.as_bytes())
                .expect("removing __AllTyps from TS failed");
            export_types_to_file::<::yerpc::Message>(&outdir.join("jsonrpc.ts"), None).expect("Failed to write TS out");

            // // Generate a raw client.
            let root_namespace = Some("T");
            let mut out = String::new();
            #(#gen_methods)*
            let ts_module = #ts_base.replace("#methods", &out);
            fs::write(&outdir.join("client.ts"), &ts_module).expect("Failed to write TS bindings");
        }
    }
}

fn qt_impl(all_types: &[TokenStream], gen_methods: &[TokenStream]) -> TokenStream {
    let qt_base = include_str!("client.hpp");
    quote! {
        /// Generate qt bindings for the JSON-RPC API.
        pub fn write_qt_bindings(outdir: &::std::path::Path, root_namespace: &str) {
            use ::yerpc::typescript::type_def::{TypeDef, type_expr::TypeInfo, DefinitionFileOptions};
            use ::yerpc::{method::Method, qt::export_types_to_file};
            use ::std::{fs, path::Path};
            use ::std::io::Write;

            // Create helper type with all exported types.
            // #(#gen_definitions)*
            #[derive(TypeDef)]
            struct __AllTyps(#(#all_types),*);
            // Write qt types to file.
            export_types_to_file::<__AllTyps>(&outdir.join("types.hpp"), root_namespace).expect("Failed to write Qt out");
            // remove __AllTyps type from output,
            // it's only used as a woraround to export all types and is not needed anymore now
            let new_content = {
                let string =
                    ::std::fs::read_to_string(&outdir.join("types.hpp")).expect("Failed to find Qt out");
                if let Some(index) = string.find("using __AllTyps = ") {
                    string[..index].to_string() + "\n}\n"
                } else {
                    panic!("did not find __AllTyps in Qt out");
                }
            };
            ::std::fs::File::create(&outdir.join("types.hpp"))
                .expect("failed to open Qt out")
                .write_all(new_content.as_bytes())
                .expect("removing __AllTyps from Qt failed");

            // // Generate a raw client.
            let mut out = String::new();
            #(#gen_methods)*
            let qt_header = #qt_base.replace("#root_namespace", root_namespace).replace("#methods", &out);
            fs::write(&outdir.join("client.hpp"), &qt_header).expect("Failed to write Qt bindings");
        }
    }
}
