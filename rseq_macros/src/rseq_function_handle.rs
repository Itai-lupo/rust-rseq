use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use syn::ItemFn;

fn make_const_string_wrapper(input: ItemFn, type_ident: syn::Type) -> TokenStream2 {
    let name = &input.sig.ident;
    let name_str = name.to_string();
    let vis = &input.vis;

    quote! {
        #[allow(non_upper_case_globals)]
        #vis const #name: #type_ident = #type_ident(#name_str);
    }
}

pub(crate) fn rseq_commit_action_impl(input: ItemFn) -> TokenStream2 {
    make_const_string_wrapper(input, syn::parse_quote!(RseqCommitActionName))
}

pub(crate) fn rseq_critical_section_start_impl(input: ItemFn) -> TokenStream2 {
    make_const_string_wrapper(input, syn::parse_quote!(RseqStart))
}

pub(crate) fn rseq_critical_section_impl(input: ItemFn) -> TokenStream2 {
    let name = &input.sig.ident;
    let vis = &input.vis;

    quote! {
        #[allow(non_upper_case_globals)]
        #vis const #name: () = ();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_expansion;
    use crate::assert_expansion_regex;
    use crate::test_utils::utils::verify_expansion;
    use syn::parse_quote;

    #[test]
    fn test_as_name_logic_basic() {
        assert_expansion!(rseq_commit_action_impl,
            fn hello() {} => "const hello : RseqCommitActionName = RseqCommitActionName (\"hello\")"
        );

        assert_expansion!(rseq_commit_action_impl,
            pub fn my_func() {} => "pub const my_func : RseqCommitActionName = RseqCommitActionName (\"my_func\")"
        );
    }

    #[test]
    fn test_as_name_logic_regex() {
        assert_expansion_regex!(rseq_commit_action_impl,
            fn hello() { println!("bye"); } => "#[.*] const hello : RseqCommitActionName = RseqCommitActionName(\".*\");"
        );

        assert_expansion_regex!(rseq_commit_action_impl,
            pub fn my_func() {} => "#[allow(non_upper_case_globals)] pub const my_func : RseqCommitActionName = RseqCommitActionName(\".*\");"
        );

        assert_expansion!(rseq_commit_action_impl,
            #[derive(Debug)]
            #[must_use]
            pub(crate) fn complex_func() {}
            => "#[allow(non_upper_case_globals)] pub(crate) const complex_func : RseqCommitActionName = RseqCommitActionName(\"complex_func\");"
        );
    }

    #[test]
    fn test_generics_and_lifetimes() {
        assert_expansion!(rseq_commit_action_impl,
            pub fn generic_handler<'a, T: std::fmt::Display>(data: &'a T) -> String {
                format!("{}", data)
            }
            => "pub const generic_handler : RseqCommitActionName = RseqCommitActionName(\"generic_handler\");"
        );
    }

    #[test]
    fn test_keywords() {
        assert_expansion!(rseq_commit_action_impl,
            pub async unsafe fn risky_async_func() {
                // some dangerous async code
                println("aa");
            }
            => "pub const risky_async_func : RseqCommitActionName = RseqCommitActionName(\"risky_async_func\");"
        );
    }
}
