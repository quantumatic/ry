//! Builds a resolution context out of a parsed AST.

use std::io;

use ry_ast::{Module, ModuleItem};
use ry_diagnostics::{BuildDiagnostic, GlobalDiagnostics};
use ry_filesystem::path_interner::{PathID, PathInterner};
use ry_fx_hash::FxHashMap;
use ry_interner::Interner;
use ry_parser::read_and_parse_module;

use crate::{
    diagnostics::{DefinitionInfo, ItemDefinedMultipleTimesDiagnostic},
    ModuleContext, ModuleItemNameBindingData,
};

/// Builds a module context from the module's AST and adds diagnostic into
/// the diagnostics if there are two definitions with the same name.
///
/// # Panics
///
/// If the interner cannot resolve any name in the module AST.
#[must_use]
pub fn build_module_context_from_ast(
    file_path_id: PathID,
    module: Module,
    interner: &Interner,
    diagnostics: &mut GlobalDiagnostics,
) -> ModuleContext {
    let mut bindings = FxHashMap::default();
    let mut implementations = vec![];
    let mut imports = vec![];

    for item in module.items {
        match item {
            ModuleItem::Impl(r#impl) => {
                implementations.push(r#impl);
            }
            ModuleItem::Import { location, path } => {
                imports.push((location, path));
            }
            _ => {
                let name = item.name_or_panic();

                if let Some(ModuleItemNameBindingData::NotAnalyzed(previous_item)) =
                    bindings.get(&name)
                {
                    diagnostics.add_file_diagnostic(
                        [file_path_id],
                        ItemDefinedMultipleTimesDiagnostic::new(
                            interner.resolve(name).unwrap(),
                            DefinitionInfo {
                                location: previous_item.location(),
                                kind: previous_item.kind(),
                            },
                            DefinitionInfo {
                                location: item.location(),
                                kind: item.kind(),
                            },
                        )
                        .build(),
                    );
                }

                bindings.insert(name, ModuleItemNameBindingData::NotAnalyzed(item));
            }
        }
    }

    ModuleContext {
        path_id: file_path_id,
        docstring: module.docstring,
        bindings,
        submodules: FxHashMap::default(),
        implementations,
        imports,
    }
}

/// Reads, parses and builds a module context.
///
/// # Panics
///
/// * If the interner cannot resolve any name in the module AST.
/// * If the file path cannot be resolved in the path storage.
///
/// # Errors
///
/// If the file cannot be read.
#[inline]
pub fn read_and_build_module_context(
    file_path_interner: &PathInterner,
    file_path_id: PathID,
    interner: &mut Interner,
    file_diagnostics: &mut GlobalDiagnostics,
) -> Result<ModuleContext, io::Error> {
    let module =
        read_and_parse_module(file_path_interner, file_path_id, file_diagnostics, interner)?;

    Ok(build_module_context_from_ast(
        file_path_id,
        module,
        interner,
        file_diagnostics,
    ))
}