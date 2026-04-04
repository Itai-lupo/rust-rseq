use proc_macro::TokenStream;
use quote::{format_ident, quote};

use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Block, ExprClosure, Ident, ItemFn, Token,
};

struct SimpleRseqInput {
    name: Ident,
    helpers: Block,
    commit: ItemFn,
    cs: ExprClosure,
}

mod rseq_function_handle;
use crate::rseq_function_handle::*;

impl Parse for SimpleRseqInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut helpers = None;
        let mut commit = None;
        let mut cs = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match key.to_string().as_str() {
                "name" => name = Some(input.parse::<Ident>()?),
                "helpers" => helpers = Some(input.parse::<Block>()?),
                "commit" => commit = Some(input.parse::<ItemFn>()?),
                "cs" => cs = Some(input.parse::<ExprClosure>()?),
                _ => return Err(syn::Error::new(key.span(), "Unknown key")),
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(SimpleRseqInput {
            name: name.ok_or(input.error("missing name"))?,
            helpers: helpers.ok_or(input.error("missing helpers"))?,
            commit: commit.ok_or(input.error("missing commit"))?,
            cs: cs.ok_or(input.error("missing cs"))?,
        })
    }
}

#[proc_macro]
pub fn rseq_context(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as SimpleRseqInput);

    let task_ident = &input.name;
    let helpers_content = &input.helpers.stmts;
    let commit_fn = &input.commit;

    let commit_name_ident = &commit_fn.sig.ident;
    let cs_logic = &input.cs;
    let entry_name = format_ident!("{}_a", task_ident);

    let helpers_str = quote! {
        #(#helpers_content)*
    }
    .to_string();

    let commit_str = quote! {
        pub unsafe extern "C" #commit_fn
    }
    .to_string();

    let cs_wrapper_str = quote! {
        pub unsafe extern "C" fn #entry_name(ctx: *mut core::ffi::c_void) -> *mut core::ffi::c_void {
            let cs_func = #cs_logic;
            cs_func(ctx)
        }
    };

    let _final_so_code = format!(
        r#"{}

 #[unsafe(no_mangle)]
#[unsafe(link_section = ".rseq_commit")]
{}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".rseq_critical")]
{}
"#,
        helpers_str, commit_str, cs_wrapper_str
    );

    // let out_dir = std::env::var("OUT_DIR").unwrap_or_else(|_| ".".into());
    // fs::write(format!("{}/rseq_{}.rs", out_dir, task_ident), final_so_code).ok();

    quote! {
        pub const #task_ident: RseqTask = RseqTask {
            main_symbol: stringify!(#entry_name),
            commit_symbol: stringify!(#commit_name_ident),
        };
    }
    .into()
}

#[proc_macro_attribute]
pub fn rseq_shared_struct(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemStruct);

    let expanded = quote! {
        #[repr(C)]
        #[derive(Clone, Copy)]
        #input
    };
    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn rseq_commit_action(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    TokenStream::from(rseq_commit_action_impl(input))
}

#[proc_macro_attribute]
pub fn rseq_critical_section_start(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    TokenStream::from(rseq_critical_section_start_impl(input))
}

#[proc_macro_attribute]
pub fn rseq_critical_section(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    TokenStream::from(rseq_critical_section_impl(input))
}

#[cfg(test)]
mod test_utils;
