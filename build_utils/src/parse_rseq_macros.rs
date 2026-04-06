use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use serde::{Deserialize, Serialize};

use snafu::prelude::*;
use syn::{Attribute, FnArg, ItemFn, ItemStruct, Type};

use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub(crate) enum RseqMacro {
    CommitAction,
    Start,
    Section,
    SharedStruct,
}

pub(crate) enum RseqItem {
    Fn(ItemFn),
    Struct(ItemStruct),
}

impl RseqMacro {
    pub(crate) fn genrate_rseq_macro_code(&self, item: RseqItem) -> Result<TokenStream2> {
        match self {
            RseqMacro::CommitAction => handle_commit(item),
            RseqMacro::Start => handle_rseq_critical_section_start(item),
            RseqMacro::Section => handle_rseq_critical_section(item),
            RseqMacro::SharedStruct => handle_struct(item),
        }
    }

    pub(crate) fn from_rust_attribure(attrs: &[Attribute]) -> Option<Self> {
        attrs.iter().find_map(|a| {
            let id = a.path().get_ident()?.to_string();
            match id.as_str() {
                "rseq_commit_action" => Some(RseqMacro::CommitAction),
                "rseq_critical_section_start" => Some(RseqMacro::Start),
                "rseq_critical_section" => Some(RseqMacro::Section),
                "rseq_shared_struct" => Some(RseqMacro::SharedStruct),
                _ => None,
            }
        })
    }
}

fn handle_commit(i: RseqItem) -> Result<TokenStream2> {
    if let RseqItem::Fn(mut f) = i {
        f.attrs
            .push(syn::parse_quote!(#[unsafe(link_section = ".rseq_commit")]));
        f.attrs.push(syn::parse_quote!(#[unsafe(no_mangle)]));
        f.attrs.push(syn::parse_quote!(#[inline(never)] ));

        Ok(quote!(#f))
    } else {
        Ok(quote!())
    }
}

fn get_param_type(arg: &FnArg) -> Option<&Type> {
    if let FnArg::Typed(arg) = arg {
        Some(&*arg.ty)
    } else {
        None
    }
}

fn as_ptr_path(ty: &Type) -> Option<&syn::Path> {
    if let Type::Ptr(ptr) = ty {
        if let Type::Path(p) = &*ptr.elem {
            return Some(&p.path);
        }
    }
    None
}

fn is_u32(ty: &syn::Type) -> bool {
    if let syn::Type::Path(tp) = ty {
        tp.path.is_ident("u32")
    } else {
        false
    }
}

fn get_result_first_arg(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(tp) = ty {
        let last_segment = tp.path.segments.last()?;

        if last_segment.ident == "Result" {
            if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                    return Some(inner_ty);
                }
            }
        }
    }
    None
}

fn assert_rseq_start_signature(f: &ItemFn) -> Result<()> {
    let sig = &f.sig;
    let fn_name = sig.ident.to_string();

    if sig.inputs.len() != 2 {
        return RseqStartWrongArgumentCountSnafu {
            name: fn_name.clone(),
            count: sig.inputs.len(),
        }
        .fail();
    }

    let arg1 = &f.sig.inputs[0];
    let ty1 = get_param_type(arg1).context(RseqStartFirstArgInvalidSnafu {
        name: fn_name.clone(),
        actual: "self".to_string(),
    })?;

    if !as_ptr_path(ty1).is_some_and(|p| p.is_ident("c_void")) {
        return RseqStartFirstArgInvalidSnafu {
            name: fn_name.clone(),
            actual: ty1.to_token_stream().to_string(),
        }
        .fail();
    }

    let arg2 = &f.sig.inputs[1];
    let ty2 = get_param_type(arg2).context(RseqStartSecondArgInvalidSnafu {
        name: fn_name.clone(),
        actual: "self".to_string(),
    })?;

    if !is_u32(ty2) {
        return RseqStartFirstArgInvalidSnafu {
            name: fn_name.clone(),
            actual: ty2.to_token_stream().to_string(),
        }
        .fail();
    }

    let return_type = match &f.sig.output {
        syn::ReturnType::Default => {
            return RseqStartReturnInvalidSnafu {
                name: fn_name.clone(),
                actual: "()".to_string(),
            }
            .fail();
        }
        syn::ReturnType::Type(_, ty) => &**ty,
    };

    let inner_ty = get_result_first_arg(return_type).context(RseqStartReturnInvalidSnafu {
        name: fn_name.clone(),
        actual: return_type.to_token_stream().to_string(),
    })?;

    if !as_ptr_path(inner_ty).is_some_and(|p| p.is_ident("c_void")) {
        return RseqStartReturnInvalidSnafu {
            name: fn_name.clone(),
            actual: return_type.to_token_stream().to_string(),
        }
        .fail();
    }

    Ok(())
}

fn handle_rseq_critical_section_start(i: RseqItem) -> Result<TokenStream2> {
    if let RseqItem::Fn(mut f) = i {
        assert_rseq_start_signature(&f).map_err(|e| e.wrap("assert_rseq_start_signature"))?;

        f.attrs.push(syn::parse_quote!(#[unsafe(no_mangle)]));
        f.attrs
            .push(syn::parse_quote!(#[unsafe(link_section = ".rseq_critical")]));
        f.vis = syn::Visibility::Public(syn::token::Pub {
            span: proc_macro2::Span::call_site(),
        });

        f.sig.unsafety = Some(syn::token::Unsafe {
            span: proc_macro2::Span::call_site(),
        });

        f.sig.abi = Some(syn::Abi {
            extern_token: syn::Token![extern](proc_macro2::Span::call_site()),
            name: Some(syn::LitStr::new("C", proc_macro2::Span::call_site())),
        });

        Ok(quote!(#f))
    } else {
        Ok(quote!())
    }
}

fn handle_rseq_critical_section(i: RseqItem) -> Result<TokenStream2> {
    if let RseqItem::Fn(mut f) = i {
        f.attrs
            .push(syn::parse_quote!(#[unsafe(link_section = ".rseq_critical")]));

        Ok(quote!(#f))
    } else {
        Ok(quote!())
    }
}
fn handle_struct(i: RseqItem) -> Result<TokenStream2> {
    if let RseqItem::Struct(mut s) = i {
        s.attrs.push(syn::parse_quote!(#[repr(C)]));
        s.attrs.push(syn::parse_quote!(#[derive(Clone, Copy)]));
        Ok(quote!(#s))
    } else {
        Ok(quote!())
    }
}
