use codegen::Scope;
use generator::generate_single_mod;
use std::fs;
use std::iter::Peekable;
use std::path::{Path, PathBuf};
use types::{Module, TypeKind};

pub fn write_mod_files<'a>(dir: &str, mut modules: Peekable<impl Iterator<Item = &'a Module>>) {
    let kind = &modules.peek().unwrap().kind;
    match kind {
        TypeKind::Type => write_types_mod(dir, modules),
        TypeKind::Method(_) => write_methods_mod(dir, modules),
        TypeKind::Enum => write_enums_mod(dir, modules),
    }
}

fn write_types_mod<'a, P: AsRef<Path>>(dir: P, modules: impl Iterator<Item = &'a Module>) {
    let mut path = dir.as_ref().to_path_buf();
    path.push("types");
    fs::create_dir_all(path.as_path()).unwrap();
    let mut string = String::new();
    let mut scope = Scope::new();
    scope.import("self::enums", "*").vis("pub");
    string.push_str("pub mod enums;\n");
    string.push_str(&scope.to_string());
    write_module_file(&path, modules, &mut string);
    path.push("mod");
    path.set_extension("rs");
    fs::write(path, string).unwrap();
}

fn write_methods_mod<'a, P: AsRef<Path>>(dir: P, modules: impl Iterator<Item = &'a Module>) {
    let mut path = dir.as_ref().to_path_buf();
    let mut string = String::new();
    path.push("methods");
    fs::create_dir_all(path.as_path()).unwrap();
    write_module_file(&path, modules, &mut string);
    path.push("mod");
    path.set_extension("rs");
    fs::write(path, string).unwrap();
}

fn write_enums_mod<'a, P: AsRef<Path>>(dir: P, modules: impl Iterator<Item = &'a Module>) {
    let mut path = dir.as_ref().to_path_buf();
    path.push("types");
    path.push("enums");
    fs::create_dir_all(path.as_path()).unwrap();
    let mut string = String::new();
    write_module_file(&path, modules, &mut string);
    path.push("mod");
    path.set_extension("rs");
    fs::write(path, string).unwrap();
}

pub fn write_module_file<'a>(
    path: &PathBuf,
    modules: impl Iterator<Item = &'a Module>,
    string: &mut String,
) {
    for module in modules {
        let mut path = path.clone();
        path.push(&module.module_name);
        path.set_extension("rs");
        fs::write(path, &module.contents).unwrap();
        generate_single_mod(module, string);
    }
}
