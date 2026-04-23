use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use syn::{
    Attribute, Expr, ExprArray, ExprCall, ExprLit, ExprMacro, ExprReference, ExprStruct, File,
    ImplItem, ImplItemFn, Item, ItemImpl, Lit, Meta, Type, Visibility,
};

pub fn validate_current_crate() {
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set"));
    validate_crate(&manifest_dir);
}

/// Generate trait method implementations from attributes.
/// Writes to OUT_DIR/dll_plugin_generated.rs
pub fn generate_metadata_code() {
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set"));
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is not set"));

    let lib_rs = manifest_dir.join("src/lib.rs");
    println!("cargo:rerun-if-changed={}", lib_rs.display());

    let source = fs::read_to_string(&lib_rs)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", lib_rs.display()));
    let file = syn::parse_file(&source)
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", lib_rs.display()));

    let default_dll_names = collect_plugin_dll_names(&file)
        .unwrap_or_else(|error| panic!("failed to collect DLL names: {error}"));

    let (attribute_exports, failures) = parse_attribute_exports(&manifest_dir, &default_dll_names)
        .unwrap_or_else(|error| panic!("failed to parse attributes: {error}"));

    if !failures.is_empty() {
        let mut message = "attribute parsing failures:".to_string();
        for failure in failures {
            message.push('\n');
            message.push_str(&failure);
        }
        panic!("{message}");
    }

    let (exports_expr, stubs_expr, partials_expr) = generate_trait_methods(&attribute_exports);
    let exports_path = out_dir.join("dll_plugin_generated.rs");
    let stubs_path = out_dir.join("dll_plugin_generated_stubs.rs");
    let partials_path = out_dir.join("dll_plugin_generated_partials.rs");

    fs::write(&exports_path, &exports_expr)
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", exports_path.display()));
    fs::write(&stubs_path, &stubs_expr)
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", stubs_path.display()));
    fs::write(&partials_path, &partials_expr)
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", partials_path.display()));
}

pub fn validate_crate(manifest_dir: &Path) {
    let lib_rs = manifest_dir.join("src/lib.rs");
    println!("cargo:rerun-if-changed={}", lib_rs.display());

    let source = fs::read_to_string(&lib_rs)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", lib_rs.display()));
    let file = syn::parse_file(&source)
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", lib_rs.display()));

    let (failures, warnings) = find_duplicate_metadata(manifest_dir, &file)
        .unwrap_or_else(|error| panic!("failed to validate {}: {error}", lib_rs.display()));

    for warning in warnings {
        println!("cargo:warning={warning}");
    }

    if failures.is_empty() {
        return;
    }

    let mut message = format!("duplicate DLL metadata detected in {}", lib_rs.display());
    for failure in failures {
        message.push_str("\n\n");
        message.push_str(&failure);
    }
    panic!("{message}");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ExportStatus {
    Implemented,
    Partial,
    Stubbed,
}

impl ExportStatus {
    fn as_label(self) -> &'static str {
        match self {
            ExportStatus::Implemented => "implemented",
            ExportStatus::Partial => "partial",
            ExportStatus::Stubbed => "stubbed",
        }
    }

    fn from_method_kind(kind: &str) -> Option<Self> {
        match kind {
            "exports" => Some(Self::Implemented),
            "partials" => Some(Self::Partial),
            "stubs" => Some(Self::Stubbed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
struct AttributeExport {
    export_name: String,
    symbol_path: String,
    dll_names: Vec<String>,
    status: ExportStatus,
    source: String,
}

#[derive(Debug, Default)]
struct ParsedExportAttributes {
    status: Option<ExportStatus>,
    export_name: Option<String>,
    dll_names: Vec<String>,
    ordinal: Option<u16>,
    has_any_export_metadata: bool,
}

fn find_duplicate_metadata(
    manifest_dir: &Path,
    file: &File,
) -> Result<(Vec<String>, Vec<String>), String> {
    let mut failures = Vec::new();
    let mut warnings = Vec::new();
    let mut manual_status_by_name: BTreeMap<String, BTreeSet<ExportStatus>> = BTreeMap::new();

    for item in &file.items {
        let Item::Impl(item_impl) = item else {
            continue;
        };

        if !is_dll_plugin_impl(item_impl) {
            continue;
        }

        let plugin_name = type_name(&item_impl.self_ty);
        let dll_names = match find_method(item_impl, "dll_names") {
            Some(method) => parse_dll_names(method)?,
            None => {
                failures.push(format!(
                    "{plugin_name}: missing dll_names() method; validator requires literal metadata lists"
                ));
                continue;
            }
        };

        let mut by_name: BTreeMap<String, Vec<&'static str>> = BTreeMap::new();
        let manual_exports = parse_exports(find_method(item_impl, "exports"), &plugin_name)?;
        record_entries(
            &mut by_name,
            &mut manual_status_by_name,
            manual_exports,
            "exports",
        );
        let manual_stubs =
            parse_named_structs(find_method(item_impl, "stubs"), "StubExport", &plugin_name)?;
        record_entries(
            &mut by_name,
            &mut manual_status_by_name,
            manual_stubs,
            "stubs",
        );
        let manual_partials = parse_named_structs(
            find_method(item_impl, "partials"),
            "PartialExport",
            &plugin_name,
        )?;
        record_entries(
            &mut by_name,
            &mut manual_status_by_name,
            manual_partials,
            "partials",
        );

        let duplicates: Vec<String> = by_name
            .into_iter()
            .filter_map(|(name, kinds)| {
                if kinds.len() < 2 {
                    return None;
                }
                Some(format!(
                    "  Duplicate metadata for \"{name}\" found in {}",
                    kinds.join(" and "),
                ))
            })
            .collect();

        if !duplicates.is_empty() {
            failures.push(format!(
                "{plugin_name} [{}]:\n{}",
                dll_names.join(", "),
                duplicates.join("\n")
            ));
        }
    }

    let default_dll_names = collect_plugin_dll_names(file)?;
    let (attribute_exports, attribute_failures) =
        parse_attribute_exports(manifest_dir, &default_dll_names)?;
    failures.extend(attribute_failures);

    let mut attr_by_dll_and_name: BTreeMap<(String, String), Vec<(ExportStatus, String)>> =
        BTreeMap::new();
    let mut attr_status_by_name: BTreeMap<String, BTreeSet<ExportStatus>> = BTreeMap::new();

    for export in &attribute_exports {
        attr_status_by_name
            .entry(export.export_name.clone())
            .or_default()
            .insert(export.status);

        for dll_name in &export.dll_names {
            attr_by_dll_and_name
                .entry((dll_name.clone(), export.export_name.clone()))
                .or_default()
                .push((export.status, export.source.clone()));
        }
    }

    for ((dll_name, export_name), entries) in attr_by_dll_and_name {
        if entries.len() < 2 {
            continue;
        }
        let statuses = entries
            .iter()
            .map(|(status, _)| status.as_label())
            .collect::<Vec<_>>()
            .join(", ");
        let sources = entries
            .iter()
            .map(|(_, source)| source.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        failures.push(format!(
            "attribute metadata duplicate export name \"{export_name}\" in DLL namespace \"{dll_name}\" (statuses: {statuses}; sources: {sources})"
        ));
    }

    for name in manual_status_by_name
        .keys()
        .chain(attr_status_by_name.keys())
        .collect::<BTreeSet<_>>()
    {
        let manual = manual_status_by_name.get(name).cloned().unwrap_or_default();
        let attributed = attr_status_by_name.get(name).cloned().unwrap_or_default();
        if manual != attributed {
            warnings.push(format!(
                "metadata divergence for export \"{name}\": manual=[{}], attributes=[{}]",
                fmt_status_set(&manual),
                fmt_status_set(&attributed)
            ));
        }
    }

    Ok((failures, warnings))
}

fn record_entries(
    by_name: &mut BTreeMap<String, Vec<&'static str>>,
    statuses_by_name: &mut BTreeMap<String, BTreeSet<ExportStatus>>,
    names: Vec<String>,
    kind: &'static str,
) {
    let status = ExportStatus::from_method_kind(kind);
    for name in names {
        by_name.entry(name.clone()).or_default().push(kind);
        if let Some(status) = status {
            statuses_by_name.entry(name).or_default().insert(status);
        }
    }
}

fn fmt_status_set(set: &BTreeSet<ExportStatus>) -> String {
    if set.is_empty() {
        return "none".to_string();
    }
    set.iter()
        .map(|status| status.as_label())
        .collect::<Vec<_>>()
        .join(",")
}

fn collect_plugin_dll_names(file: &File) -> Result<Vec<String>, String> {
    let mut dll_names = BTreeSet::new();
    for item in &file.items {
        let Item::Impl(item_impl) = item else {
            continue;
        };
        if !is_dll_plugin_impl(item_impl) {
            continue;
        }
        if let Some(method) = find_method(item_impl, "dll_names") {
            for dll_name in parse_dll_names(method)? {
                dll_names.insert(dll_name);
            }
        }
    }

    Ok(dll_names.into_iter().collect())
}

fn parse_attribute_exports(
    manifest_dir: &Path,
    default_dll_names: &[String],
) -> Result<(Vec<AttributeExport>, Vec<String>), String> {
    let mut exports = Vec::new();
    let mut failures = Vec::new();
    let src_dir = manifest_dir.join("src");
    if !src_dir.exists() {
        return Ok((exports, failures));
    }

    for file_path in collect_rs_files(&src_dir)? {
        println!("cargo:rerun-if-changed={}", file_path.display());
        let source = fs::read_to_string(&file_path)
            .map_err(|error| format!("failed to read {}: {error}", file_path.display()))?;
        let parsed = syn::parse_file(&source)
            .map_err(|error| format!("failed to parse {}: {error}", file_path.display()))?;
        let module_prefix = module_prefix_for_file(&src_dir, &file_path)?;

        for item in parsed.items {
            match item {
                Item::Fn(item_fn) => parse_attributed_fn(
                    &file_path,
                    &module_prefix,
                    &item_fn,
                    default_dll_names,
                    &mut exports,
                    &mut failures,
                ),
                Item::Static(item_static) => parse_attributed_static(
                    &file_path,
                    &module_prefix,
                    &item_static,
                    default_dll_names,
                    &mut exports,
                    &mut failures,
                ),
                _ => {}
            }
        }
    }

    Ok((exports, failures))
}

fn module_prefix_for_file(src_dir: &Path, file_path: &Path) -> Result<String, String> {
    let rel_path = file_path.strip_prefix(src_dir).map_err(|error| {
        format!(
            "failed to derive module path for {}: {error}",
            file_path.display()
        )
    })?;

    let mut parts: Vec<String> = rel_path
        .iter()
        .map(|part| part.to_string_lossy().to_string())
        .collect();

    if let Some(last) = parts.last_mut()
        && last.ends_with(".rs")
    {
        *last = last.trim_end_matches(".rs").to_string();
    }

    if parts == ["lib".to_string()] {
        return Ok(String::new());
    }

    if parts.last().is_some_and(|p| p == "mod") {
        parts.pop();
    }

    Ok(parts.join("::"))
}

fn parse_attributed_fn(
    file_path: &Path,
    module_prefix: &str,
    item_fn: &syn::ItemFn,
    default_dll_names: &[String],
    exports: &mut Vec<AttributeExport>,
    failures: &mut Vec<String>,
) {
    let source = format!("{}::{}", file_path.display(), item_fn.sig.ident);
    let attrs = match parse_export_attributes(&item_fn.attrs) {
        Ok(attrs) => attrs,
        Err(error) => {
            failures.push(format!("{source}: {error}"));
            return;
        }
    };

    let has_metadata = attrs.has_any_export_metadata;
    let has_status = attrs.status.is_some();
    if has_metadata && !has_status {
        failures.push(format!(
            "{source}: missing status attribute (expected one of #[implemented], #[partial], #[stubbed])"
        ));
        return;
    }
    if !has_status {
        return;
    }

    if !matches!(item_fn.vis, Visibility::Public(_)) {
        failures.push(format!(
            "{source}: status attributes are only valid on pub exported items"
        ));
        return;
    }

    let dll_names = if attrs.dll_names.is_empty() {
        default_dll_names.to_vec()
    } else {
        attrs.dll_names
    };
    if dll_names.is_empty() {
        failures.push(format!(
            "{source}: could not resolve DLL namespace; add #[dll(\"...\")] or keep dll_names() metadata"
        ));
        return;
    }

    let export_name = attrs
        .export_name
        .unwrap_or_else(|| item_fn.sig.ident.to_string());
    let symbol_path = if module_prefix.is_empty() {
        item_fn.sig.ident.to_string()
    } else {
        format!("{module_prefix}::{}", item_fn.sig.ident)
    };

    exports.push(AttributeExport {
        export_name,
        symbol_path,
        dll_names,
        status: attrs.status.expect("checked above"),
        source,
    });
}

fn parse_attributed_static(
    file_path: &Path,
    module_prefix: &str,
    item_static: &syn::ItemStatic,
    default_dll_names: &[String],
    exports: &mut Vec<AttributeExport>,
    failures: &mut Vec<String>,
) {
    let source = format!("{}::{}", file_path.display(), item_static.ident);
    let attrs = match parse_export_attributes(&item_static.attrs) {
        Ok(attrs) => attrs,
        Err(error) => {
            failures.push(format!("{source}: {error}"));
            return;
        }
    };

    let has_metadata = attrs.has_any_export_metadata;
    let has_status = attrs.status.is_some();
    if has_metadata && !has_status {
        failures.push(format!(
            "{source}: missing status attribute (expected one of #[implemented], #[partial], #[stubbed])"
        ));
        return;
    }
    if !has_status {
        return;
    }

    if attrs.ordinal.is_some() {
        failures.push(format!(
            "{source}: #[ordinal(...)] is only valid on function exports"
        ));
    }

    if !matches!(item_static.vis, Visibility::Public(_)) {
        failures.push(format!(
            "{source}: status attributes are only valid on pub exported items"
        ));
        return;
    }

    let dll_names = if attrs.dll_names.is_empty() {
        default_dll_names.to_vec()
    } else {
        attrs.dll_names
    };
    if dll_names.is_empty() {
        failures.push(format!(
            "{source}: could not resolve DLL namespace; add #[dll(\"...\")] or keep dll_names() metadata"
        ));
        return;
    }

    let export_name = attrs
        .export_name
        .unwrap_or_else(|| item_static.ident.to_string());
    let symbol_path = if module_prefix.is_empty() {
        item_static.ident.to_string()
    } else {
        format!("{module_prefix}::{}", item_static.ident)
    };

    exports.push(AttributeExport {
        export_name,
        symbol_path,
        dll_names,
        status: attrs.status.expect("checked above"),
        source,
    });
}

fn collect_rs_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for entry in
        fs::read_dir(dir).map_err(|error| format!("failed to read {}: {error}", dir.display()))?
    {
        let entry =
            entry.map_err(|error| format!("failed to iterate {}: {error}", dir.display()))?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(collect_rs_files(&path)?);
            continue;
        }
        if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn parse_export_attributes(attrs: &[Attribute]) -> Result<ParsedExportAttributes, String> {
    let mut parsed = ParsedExportAttributes::default();

    for attr in attrs {
        let attr_name = attr
            .path()
            .segments
            .last()
            .map(|segment| segment.ident.to_string());

        if attr_name.as_deref() == Some("implemented") {
            parsed.has_any_export_metadata = true;
            set_status(&mut parsed, ExportStatus::Implemented)?;
            continue;
        }
        if attr_name.as_deref() == Some("partial") {
            parsed.has_any_export_metadata = true;
            set_status(&mut parsed, ExportStatus::Partial)?;
            continue;
        }
        if attr_name.as_deref() == Some("stubbed") {
            parsed.has_any_export_metadata = true;
            set_status(&mut parsed, ExportStatus::Stubbed)?;
            continue;
        }
        if attr_name.as_deref() == Some("dll") {
            parsed.has_any_export_metadata = true;
            let dll_name = attr
                .parse_args::<syn::LitStr>()
                .map_err(|error| format!("invalid #[dll(...)] attribute: {error}"))?
                .value();
            parsed.dll_names.push(dll_name);
            continue;
        }
        if attr_name.as_deref() == Some("ordinal") {
            parsed.has_any_export_metadata = true;
            let ordinal = attr
                .parse_args::<syn::LitInt>()
                .map_err(|error| format!("invalid #[ordinal(...)] attribute: {error}"))?
                .base10_parse::<u16>()
                .map_err(|error| format!("invalid #[ordinal(...)] value: {error}"))?;
            if parsed.ordinal.replace(ordinal).is_some() {
                return Err("duplicate #[ordinal(...)] attribute".to_string());
            }
            continue;
        }
        if attr_name.as_deref() == Some("export_name") {
            parsed.has_any_export_metadata = true;
            let Meta::NameValue(name_value) = &attr.meta else {
                return Err("invalid #[export_name = \"...\"] attribute".to_string());
            };
            let Expr::Lit(ExprLit {
                lit: Lit::Str(export_name),
                ..
            }) = &name_value.value
            else {
                return Err("invalid #[export_name = \"...\"] attribute".to_string());
            };
            if parsed.export_name.replace(export_name.value()).is_some() {
                return Err("duplicate #[export_name = \"...\"] attribute".to_string());
            }
        }
    }

    Ok(parsed)
}

fn set_status(parsed: &mut ParsedExportAttributes, status: ExportStatus) -> Result<(), String> {
    if let Some(previous) = parsed.status
        && previous != status
    {
        return Err(format!(
            "multiple status attributes are not allowed: {} and {}",
            previous.as_label(),
            status.as_label()
        ));
    }
    parsed.status = Some(status);
    Ok(())
}

fn is_dll_plugin_impl(item_impl: &ItemImpl) -> bool {
    let Some((_, path, _)) = &item_impl.trait_ else {
        return false;
    };
    path.segments
        .last()
        .is_some_and(|segment| segment.ident == "DllPlugin")
}

fn find_method<'a>(item_impl: &'a ItemImpl, name: &str) -> Option<&'a ImplItemFn> {
    item_impl.items.iter().find_map(|item| {
        let ImplItem::Fn(method) = item else {
            return None;
        };
        (method.sig.ident == name).then_some(method)
    })
}

fn type_name(ty: &Type) -> String {
    match ty {
        Type::Path(path) => path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
            .unwrap_or_else(|| "<unknown>".to_string()),
        _ => "<unknown>".to_string(),
    }
}

fn parse_dll_names(method: &ImplItemFn) -> Result<Vec<String>, String> {
    let expr = tail_expr(method)
        .ok_or_else(|| format!("{}(): expected a tail expression", method.sig.ident))?;
    let Expr::Reference(ExprReference { expr, .. }) = expr else {
        return Err(format!(
            "{}(): expected a literal slice reference",
            method.sig.ident
        ));
    };
    let Expr::Array(ExprArray { elems, .. }) = expr.as_ref() else {
        return Err(format!(
            "{}(): expected a literal string slice",
            method.sig.ident
        ));
    };

    elems
        .iter()
        .map(parse_string_literal)
        .collect::<Result<Vec<_>, _>>()
}

fn parse_exports(method: Option<&ImplItemFn>, plugin_name: &str) -> Result<Vec<String>, String> {
    let Some(method) = method else {
        return Err(format!("{plugin_name}: missing exports() method"));
    };

    let entries = parse_vec_expr(method, "exports")?;
    let mut names = Vec::new();

    for entry in entries {
        let Expr::Call(ExprCall { func, args, .. }) = entry else {
            return Err(format!(
                "{plugin_name}: exports() entries must be Export::* calls"
            ));
        };

        let Expr::Path(path) = func.as_ref() else {
            return Err(format!(
                "{plugin_name}: exports() entries must call Export::*"
            ));
        };

        let Some(kind) = path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
        else {
            return Err(format!("{plugin_name}: could not read export kind"));
        };

        match kind.as_str() {
            "Func" | "Data" => {
                let Some(first_arg) = args.first() else {
                    return Err(format!("{plugin_name}: {kind} export is missing its name"));
                };
                names.push(parse_string_literal(first_arg)?);
            }
            "Ordinal" => {}
            _ => {
                return Err(format!(
                    "{plugin_name}: unsupported export entry kind `{kind}` in exports()"
                ));
            }
        }
    }

    Ok(names)
}

fn parse_named_structs(
    method: Option<&ImplItemFn>,
    expected_struct: &str,
    plugin_name: &str,
) -> Result<Vec<String>, String> {
    let Some(method) = method else {
        return Ok(Vec::new());
    };

    let entries = parse_vec_expr(method, expected_struct)?;
    let mut names = Vec::new();

    for entry in entries {
        let Expr::Struct(ExprStruct { path, fields, .. }) = entry else {
            return Err(format!(
                "{plugin_name}: {}() entries must be {expected_struct} structs",
                method.sig.ident
            ));
        };

        let Some(struct_name) = path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
        else {
            return Err(format!(
                "{plugin_name}: could not read struct name in {}()",
                method.sig.ident
            ));
        };
        if struct_name != expected_struct {
            return Err(format!(
                "{plugin_name}: {}() entries must be {expected_struct}, found {struct_name}",
                method.sig.ident
            ));
        }

        let Some(name_field) = fields
            .iter()
            .find(|field| matches!(&field.member, syn::Member::Named(ident) if ident == "name"))
        else {
            return Err(format!(
                "{plugin_name}: {expected_struct} is missing a name field"
            ));
        };
        names.push(parse_string_literal(&name_field.expr)?);
    }

    Ok(names)
}

fn parse_vec_expr(method: &ImplItemFn, context: &str) -> Result<Vec<Expr>, String> {
    let expr = tail_expr(method)
        .ok_or_else(|| format!("{}(): expected a vec![] tail expression", method.sig.ident))?;
    let Expr::Macro(ExprMacro { mac, .. }) = expr else {
        return Err(format!("{}(): expected vec![]", method.sig.ident));
    };

    if !mac.path.is_ident("vec") {
        return Err(format!("{}(): expected vec![]", method.sig.ident));
    }

    let array_source = format!("[{}]", mac.tokens);
    let array = syn::parse_str::<ExprArray>(&array_source).map_err(|error| {
        format!(
            "{}(): failed to parse {context} metadata vector literal: {error}",
            method.sig.ident
        )
    })?;

    Ok(array.elems.into_iter().collect())
}

fn tail_expr(method: &ImplItemFn) -> Option<&Expr> {
    method.block.stmts.last().and_then(|stmt| match stmt {
        syn::Stmt::Expr(expr, _) => Some(expr),
        _ => None,
    })
}

fn parse_string_literal(expr: &Expr) -> Result<String, String> {
    let Expr::Lit(ExprLit {
        lit: Lit::Str(lit), ..
    }) = expr
    else {
        return Err("expected a string literal".to_string());
    };
    Ok(lit.value())
}

/// Generate Rust expression snippets for trait method bodies from collected attributes.
fn generate_trait_methods(exports: &[AttributeExport]) -> (String, String, String) {
    let mut exports_expr = String::from("vec![\n");
    let mut stubs_expr = String::from("vec![\n");
    let mut partials_expr = String::from("vec![\n");

    // Collect by status
    let mut implemented: Vec<_> = exports
        .iter()
        .filter(|e| e.status == ExportStatus::Implemented)
        .collect();
    let mut partials: Vec<_> = exports
        .iter()
        .filter(|e| e.status == ExportStatus::Partial)
        .collect();
    let mut stubs: Vec<_> = exports
        .iter()
        .filter(|e| e.status == ExportStatus::Stubbed)
        .collect();

    // Sort by name for consistency
    implemented.sort_by(|a, b| a.export_name.cmp(&b.export_name));
    partials.sort_by(|a, b| a.export_name.cmp(&b.export_name));
    stubs.sort_by(|a, b| a.export_name.cmp(&b.export_name));

    for export in &implemented {
        exports_expr.push_str(&format!(
            "    rine_dlls::Export::Func(\"{}\", as_win_api!({})),\n",
            export.export_name, export.symbol_path
        ));
    }
    exports_expr.push(']');

    for export in &stubs {
        stubs_expr.push_str(&format!(
            "    rine_dlls::StubExport {{ name: \"{}\", func: as_win_api!({}) }},\n",
            export.export_name, export.symbol_path
        ));
    }
    stubs_expr.push(']');

    for export in &partials {
        partials_expr.push_str(&format!(
            "    rine_dlls::PartialExport {{ name: \"{}\", func: as_win_api!({}) }},\n",
            export.export_name, export.symbol_path
        ));
    }
    partials_expr.push(']');

    (exports_expr, stubs_expr, partials_expr)
}

#[cfg(test)]
mod tests {
    use super::{ExportStatus, parse_export_attributes};

    #[test]
    fn parses_status_and_export_name() {
        let attrs = vec![
            syn::parse_quote!(#[implemented]),
            syn::parse_quote!(#[export_name = "printf"]),
            syn::parse_quote!(#[dll("msvcrt.dll")]),
            syn::parse_quote!(#[ordinal(12)]),
        ];

        let parsed = parse_export_attributes(&attrs).expect("attributes should parse");
        assert_eq!(parsed.status, Some(ExportStatus::Implemented));
        assert_eq!(parsed.export_name.as_deref(), Some("printf"));
        assert_eq!(parsed.dll_names, vec!["msvcrt.dll"]);
        assert_eq!(parsed.ordinal, Some(12));
    }

    #[test]
    fn rejects_conflicting_statuses() {
        let attrs = vec![
            syn::parse_quote!(#[implemented]),
            syn::parse_quote!(#[stubbed]),
        ];

        let error = parse_export_attributes(&attrs).expect_err("should fail");
        assert!(error.contains("multiple status attributes"));
    }

    #[test]
    fn rejects_duplicate_export_name() {
        let attrs = vec![
            syn::parse_quote!(#[export_name = "a"]),
            syn::parse_quote!(#[export_name = "b"]),
        ];

        let error = parse_export_attributes(&attrs).expect_err("should fail");
        assert!(error.contains("duplicate #[export_name"));
    }
}
