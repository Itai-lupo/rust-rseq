use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Path, PathBuf},
    time::SystemTime,
};
use syn::{
    visit::{self, Visit},
    Attribute, ItemFn, ItemStruct,
};
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RseqMacro {
    CommitAction,
    Start,
    Section,
    SharedStruct,
}

pub enum RseqItem {
    Fn(ItemFn),
    Struct(ItemStruct),
}

#[derive(Serialize, Deserialize)]
struct CacheEntry {
    path: PathBuf,
    mtime: u64,
    found: Vec<(String, RseqMacro, String)>,
}

pub struct RseqProcessor {
    cache: Vec<CacheEntry>,
    pub items: Vec<(RseqItem, RseqMacro)>,
    cache_path: PathBuf,
}

impl RseqProcessor {
    pub fn new() -> Self {
        let out_dir = env::var("OUT_DIR").unwrap();
        let cache_path = Path::new(&out_dir).join("rseq_cache.json");

        let cache = fs::read_to_string(&cache_path)
            .ok()
            .and_then(|s| {
                let c: Result<Vec<CacheEntry>, _> = serde_json::from_str(&s);
                c.ok()
            })
            .unwrap_or_default();

        Self {
            cache,
            items: Vec::new(),
            cache_path,
        }
    }

    pub fn scan_all(&mut self) {
        let metadata_result = cargo_metadata::MetadataCommand::new().exec();
        let metadata = match metadata_result {
            Ok(m) => m,
            Err(_) => return,
        };

        self.cache.retain(|e| e.path.exists());

        let workspace_members = &metadata.workspace_members;

        for package in metadata
            .packages
            .iter()
            .filter(|p| workspace_members.contains(&p.id))
        {
            if let Some(root) = package.manifest_path.parent() {
                let root_path = root.to_path_buf();

                let src = root_path.join("src");
                if src.exists() {
                    self.scan_folder(src.as_std_path());
                }

                let tests = root_path.join("tests");
                if tests.exists() {
                    self.scan_folder(tests.as_std_path());
                }
            }
        }
        if let Ok(s) = serde_json::to_string(&self.cache) {
            let _ = fs::write(&self.cache_path, s);
        }
    }

    fn scan_folder(&mut self, root: &Path) {
        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "rs") {
                let mtime = fs::metadata(path)
                    .and_then(|m| m.modified())
                    .map(|t| {
                        t.duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                    })
                    .unwrap_or(0);

                if let Some(e) = self
                    .cache
                    .iter()
                    .find(|e| e.path == path && e.mtime >= mtime)
                {
                    for (code, m, _name) in &e.found {
                        let item = if *m == RseqMacro::SharedStruct {
                            RseqItem::Struct(
                                syn::parse_str(code).expect("Cache corruption (struct)"),
                            )
                        } else {
                            RseqItem::Fn(syn::parse_str(code).expect("Cache corruption (fn)"))
                        };
                        self.items.push((item, *m));
                    }
                    println!("cargo:rerun-if-changed={}", path.display());
                    continue;
                }

                let content = fs::read_to_string(path).unwrap_or_default();
                if content.contains("rseq_") {
                    if let Ok(file) = syn::parse_file(&content) {
                        let mut found_in_file = Vec::new();

                        let mut v = RseqVisitor { items: Vec::new() };
                        v.visit_file(&file);

                        for (item, m) in v.items {
                            let name = match &item {
                                RseqItem::Fn(f) => f.sig.ident.to_string(),
                                RseqItem::Struct(s) => s.ident.to_string(),
                            };

                            let code = match &item {
                                RseqItem::Fn(f) => f.to_token_stream().to_string(),
                                RseqItem::Struct(s) => s.to_token_stream().to_string(),
                            };
                            found_in_file.push((code, m, name));
                            self.items.push((item, m));
                        }

                        self.cache.retain(|e| e.path != path);
                        self.cache.push(CacheEntry {
                            path: path.to_path_buf(),
                            mtime,
                            found: found_in_file,
                        });
                    }
                }
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }

    pub fn generate_file_content(&mut self) -> String {
        let mut collected = Vec::new();
        for (item, m_type) in std::mem::take(&mut self.items) {
            let tokens = match m_type {
                RseqMacro::CommitAction => handle_commit(item),
                RseqMacro::Start => handle_start(item),
                RseqMacro::Section => handle_section(item),
                RseqMacro::SharedStruct => handle_struct(item),
            };
            collected.push(tokens);
        }

        let res = quote! { #(#collected)*  };
        let syntax_tree = syn::parse2(res).expect("Generated invalid code");
        prettyplease::unparse(&syntax_tree)
    }
}

struct RseqVisitor {
    items: Vec<(RseqItem, RseqMacro)>,
}

impl<'ast> Visit<'ast> for RseqVisitor {
    fn visit_item_fn(&mut self, i: &'ast ItemFn) {
        if let Some(m) = get_rseq_macro(&i.attrs) {
            let mut clean = i.clone();
            clean
                .attrs
                .retain(|a| get_rseq_macro(std::slice::from_ref(a)).is_none());
            self.items.push((RseqItem::Fn(clean), m));
        }
        visit::visit_item_fn(self, i);
    }
    fn visit_item_struct(&mut self, i: &'ast ItemStruct) {
        if let Some(m) = get_rseq_macro(&i.attrs) {
            let mut clean = i.clone();
            clean
                .attrs
                .retain(|a| get_rseq_macro(std::slice::from_ref(a)).is_none());
            self.items.push((RseqItem::Struct(clean), m));
        }
        visit::visit_item_struct(self, i);
    }
}

fn handle_commit(i: RseqItem) -> TokenStream2 {
    if let RseqItem::Fn(mut f) = i {
        f.attrs
            .push(syn::parse_quote!(#[unsafe(link_section = ".rseq_commit")]));
        f.attrs.push(syn::parse_quote!(#[unsafe(no_mangle)]));
        f.attrs.push(syn::parse_quote!(#[inline(never)] ));

        quote!(#f)
    } else {
        quote!()
    }
}
fn handle_start(i: RseqItem) -> TokenStream2 {
    if let RseqItem::Fn(mut f) = i {
        f.attrs.push(syn::parse_quote!(#[unsafe(no_mangle)]));
        f.attrs
            .push(syn::parse_quote!(#[unsafe(link_section = ".rseq_critical")]));

        quote!(#f)
    } else {
        quote!()
    }
}
fn handle_section(i: RseqItem) -> TokenStream2 {
    if let RseqItem::Fn(mut f) = i {
        f.attrs
            .push(syn::parse_quote!(#[unsafe(link_section = ".rseq_critical")]));

        quote!(#f)
    } else {
        quote!()
    }
}
fn handle_struct(i: RseqItem) -> TokenStream2 {
    if let RseqItem::Struct(mut s) = i {
        s.attrs.push(syn::parse_quote!(#[repr(C)]));
        s.attrs.push(syn::parse_quote!(#[derive(Clone, Copy)]));
        quote!(#s)
    } else {
        quote!()
    }
}

fn get_rseq_macro(attrs: &[Attribute]) -> Option<RseqMacro> {
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

pub fn genrate_rseq_code() -> PathBuf {
    let mut processor = RseqProcessor::new();

    processor.scan_all();

    let content = processor.generate_file_content();

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest_path = Path::new(&out_dir).join("rseq_gen.rs");

    fs::write(&dest_path, content).expect("Failed to write rseq_gen.rs");
    dest_path
}
