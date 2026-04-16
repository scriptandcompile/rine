use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use syn::{
    Expr, ExprArray, ExprCall, ExprLit, ExprMacro, ExprReference, ExprStruct, File, ImplItem,
    ImplItemFn, Item, ItemImpl, Lit, Type,
};

pub fn validate_current_crate() {
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set"));
    validate_crate(&manifest_dir);
}

pub fn validate_crate(manifest_dir: &Path) {
    let lib_rs = manifest_dir.join("src/lib.rs");
    println!("cargo:rerun-if-changed={}", lib_rs.display());

    let source = fs::read_to_string(&lib_rs)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", lib_rs.display()));
    let file = syn::parse_file(&source)
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", lib_rs.display()));

    let failures = find_duplicate_metadata(&file)
        .unwrap_or_else(|error| panic!("failed to validate {}: {error}", lib_rs.display()));

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

fn find_duplicate_metadata(file: &File) -> Result<Vec<String>, String> {
    let mut failures = Vec::new();

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
        record_entries(
            &mut by_name,
            parse_exports(find_method(item_impl, "exports"), &plugin_name)?,
            "implemented",
        );
        record_entries(
            &mut by_name,
            parse_named_structs(find_method(item_impl, "stubs"), "StubExport", &plugin_name)?,
            "stub",
        );
        record_entries(
            &mut by_name,
            parse_named_structs(
                find_method(item_impl, "partials"),
                "PartialExport",
                &plugin_name,
            )?,
            "partial",
        );

        let duplicates: Vec<String> = by_name
            .into_iter()
            .filter_map(|(name, kinds)| {
                if kinds.len() < 2 {
                    return None;
                }
                Some(format!("- {name}: {}", kinds.join(", ")))
            })
            .collect();

        if !duplicates.is_empty() {
            failures.push(format!(
                "{plugin_name} [{}]\n{}",
                dll_names.join(", "),
                duplicates.join("\n")
            ));
        }
    }

    Ok(failures)
}

fn record_entries(
    by_name: &mut BTreeMap<String, Vec<&'static str>>,
    names: Vec<String>,
    kind: &'static str,
) {
    for name in names {
        by_name.entry(name).or_default().push(kind);
    }
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
