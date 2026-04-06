use quote::{quote, ToTokens};
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Path, PathBuf},
    time::SystemTime,
};
use walkdir::WalkDir;

use syn::{
    visit::{self, Visit},
    ItemFn, ItemStruct,
};

use {snafu::ResultExt, snafu::OptionExt}; 

use crate::{JsonSnafu, parse_rseq_macros::RseqItem};
use crate::parse_rseq_macros::RseqMacro;
use crate::{Result, RseqBuildError, CargoMetadataSnafu, IoSnafu};

pub fn genrate_rseq_code() -> Result<PathBuf> {
    let mut processor = RseqProcessor::new()?;

    processor.scan_all()?;

    let content = processor.generate_file_content()?;

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest_path = Path::new(&out_dir).join("rseq_gen.rs");

    fs::write(&dest_path, content).expect("Failed to write rseq_gen.rs");
    Ok(dest_path)
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
    pub fn new() -> Result<Self> {
        let out_dir = env::var("OUT_DIR").unwrap();
        let cache_path: PathBuf = Path::new(&out_dir).join("rseq_cache.json");

        Ok(Self {
            cache: RseqProcessor::load_cache(&cache_path)?,
            items: Vec::new(),
            cache_path: cache_path,
        })
    }

    pub fn load_cache(cache_path: &PathBuf) -> Result<Vec<CacheEntry>> {
        let content = match std::fs::read_to_string(cache_path) {
            Ok(content) => content,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => {
                return Err(e).context(IoSnafu)?;
            }
        };

        let cache: Vec<CacheEntry> = serde_json::from_str(&content).context(JsonSnafu{})?;
        // .wrap_err("Failed to parse rseq macro parse cache as JSON")?;
        Ok(cache)
    }

    pub fn scan_all(&mut self) -> Result<()> {
        let metadata_result = cargo_metadata::MetadataCommand::new().exec();
        let metadata = metadata_result.context(CargoMetadataSnafu)?;

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
                    self.scan_folder(src.as_std_path())?;
                }

                let tests = root_path.join("tests");
                if tests.exists() {
                    self.scan_folder(tests.as_std_path())?;
                }
            }
        }
        if let Ok(s) = serde_json::to_string(&self.cache) {
            let _ = fs::write(&self.cache_path, s);
        }

        Ok(())
    }

    fn scan_folder(&mut self, root: &Path) -> Result<()> {
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

                        let mut v: FindRseqMacrosVisitor =
                            FindRseqMacrosVisitor { items: Vec::new() };
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

        Ok(())
    }

    pub fn generate_file_content(&mut self) -> Result<String> {
        let mut collected = Vec::new();
        for (item, m_type) in std::mem::take(&mut self.items) {
            let tokens = m_type.genrate_rseq_macro_code(item)?;
            collected.push(tokens);
        }

        let res = quote! { #(#collected)*  };
        let syntax_tree = syn::parse2(res).expect("Generated invalid code");
        Ok(prettyplease::unparse(&syntax_tree))
    }
}

struct FindRseqMacrosVisitor {
    items: Vec<(RseqItem, RseqMacro)>,
}

macro_rules! strip_rseq_attrs {
    ($item:expr) => {{
        let mut cleaned = $item.clone();
        cleaned
            .attrs
            .retain(|a| RseqMacro::from_rust_attribure(std::slice::from_ref(a)).is_none());
        cleaned
    }};
}

impl<'ast> Visit<'ast> for FindRseqMacrosVisitor {
    fn visit_item_fn(&mut self, i: &'ast ItemFn) {
        if let Some(m) = RseqMacro::from_rust_attribure(&i.attrs) {
            let clean: ItemFn = strip_rseq_attrs!(i);
            self.items.push((RseqItem::Fn(clean), m));
        }
        visit::visit_item_fn(self, i);
    }

    fn visit_item_struct(&mut self, i: &'ast ItemStruct) {
        if let Some(m) = RseqMacro::from_rust_attribure(&i.attrs) {
            let clean: ItemStruct = strip_rseq_attrs!(i);
            self.items.push((RseqItem::Struct(clean), m));
        }
        visit::visit_item_struct(self, i);
    }
}
