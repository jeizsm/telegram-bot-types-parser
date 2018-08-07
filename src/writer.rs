use codegen::{Field, Scope};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use types::{Kind, Module, RustEnum, RustStruct, RustType};
use utils::{camel_case, snake_case};

pub trait Writer {
    fn write<P: AsRef<Path>>(self, dir: P, modules: &mut HashSet<Module>);
}

impl Writer for RustStruct {
    fn write<P: AsRef<Path>>(self, dir: P, modules: &mut HashSet<Module>) {
        let mut path = dir.as_ref().to_path_buf();
        match self.kind {
            Kind::Type => path.push("types"),
            Kind::Method => path.push("methods"),
            _ => panic!("wrong kind"),
        };
        fs::create_dir_all(path.as_path()).unwrap();
        let file_name = snake_case(&self.name);
        path.push(&file_name);
        path.set_extension("rs");
        let mut scope = Scope::new();
        {
            match self.kind {
                Kind::Type => scope.import("super", "*"),
                Kind::Method => scope.import("super", "*"),
                _ => panic!("wrong kind"),
            };
            let new_struct = scope
                .new_struct(&self.name)
                .doc(&self.docs.join("\n"))
                .derive("Serialize")
                .derive("Deserialize")
                .derive("Debug")
                .vis("pub");
            for field in self.fields {
                let field_type = match field.rust_type {
                    RustType::String(rust_type) => rust_type,
                    RustType::Enum(enum_type) => {
                        let rust_type = if enum_type.is_array {
                            format!("Vec<{}>", camel_case(&field.name))
                        } else {
                            camel_case(&field.name)
                        };
                        let rust_type = if enum_type.is_optional {
                            format!("Option<{}>", rust_type)
                        } else {
                            rust_type
                        };
                        enum_type.write(dir.as_ref(), modules);
                        rust_type
                    }
                };
                let mut rust_field = match (field.name.as_ref(), field_type) {
                    ("type", field_type) => {
                        let mut field = Field::new("type_", field_type);
                        field.annotation(vec![r#"#[serde(rename = "type")]"#]);
                        field
                    }
                    (name, field_type) => Field::new(name, field_type),
                };
                rust_field.vis("pub").doc(&field.doc);
                new_struct.push_field(rust_field);
            }
        }
        fs::write(path, scope.to_string()).unwrap();
        let module = Module {
            kind: self.kind,
            module_name: file_name,
            module_type: self.name,
        };
        modules.insert(module);
    }
}

impl Writer for RustEnum {
    fn write<P: AsRef<Path>>(self, dir: P, modules: &mut HashSet<Module>) {
        let mut path = dir.as_ref().to_path_buf();
        path.push("types");
        path.push("enums");
        fs::create_dir_all(path.as_path()).unwrap();
        let file_name = snake_case(&self.name);
        path.push(&file_name);
        path.set_extension("rs");
        let mut scope = Scope::new();
        {
            scope.import("super", "*");
            let rust_enum = scope
                .new_enum(&self.name)
                .derive("Serialize")
                .derive("Deserialize")
                .derive("Debug")
                .vis("pub");
            for string_variant in self.variants {
                let variant = rust_enum.new_variant(&string_variant);
                variant.tuple(&string_variant);
            }
        }
        fs::write(path, scope.to_string()).unwrap();
        let module = Module {
            kind: Kind::Enum,
            module_name: file_name,
            module_type: self.name,
        };
        modules.insert(module);
    }
}

pub fn write_mod_files<'a>(dir: &str, kind: &Kind, modules: impl Iterator<Item = &'a Module>) {
    match *kind {
        Kind::Type => write_types_mod(dir, modules),
        Kind::Method => write_methods_mod(dir, modules),
        Kind::Enum => write_enums_mod(dir, modules),
    }
}

fn write_types_mod<'a, P: AsRef<Path>>(dir: P, modules: impl Iterator<Item = &'a Module>) {
    let mut path = dir.as_ref().to_path_buf();
    path.push("types");
    path.push("mod");
    path.set_extension("rs");
    let mut string = parse_all_mod(modules);
    let mut scope = Scope::new();
    scope
        .new_struct("Integer")
        .derive("Serialize")
        .derive("Deserialize")
        .derive("Debug")
        .tuple_field("i64")
        .vis("pub");
    scope
        .new_struct("True")
        .derive("Serialize")
        .derive("Deserialize")
        .derive("Debug")
        .tuple_field("bool")
        .vis("pub");
    scope
        .new_struct("Float")
        .derive("Serialize")
        .derive("Deserialize")
        .derive("Debug")
        .tuple_field("f64")
        .vis("pub");
    scope
        .new_struct("CallbackGame")
        .derive("Serialize")
        .derive("Deserialize")
        .derive("Debug")
        .vis("pub");
    scope
        .new_struct("InputFile")
        .derive("Serialize")
        .derive("Deserialize")
        .derive("Debug")
        .tuple_field("String")
        .vis("pub");
    scope.import("self::enums", "*").vis("pub");
    string.push_str("pub mod enums;\n");
    string.push_str(&scope.to_string());
    fs::write(path, string).unwrap();
}

fn write_methods_mod<'a, P: AsRef<Path>>(dir: P, modules: impl Iterator<Item = &'a Module>) {
    let mut path = dir.as_ref().to_path_buf();
    let mut string = parse_all_mod(modules);
    let mut scope = Scope::new();
    scope.import("types", "*");
    string.push_str(&scope.to_string());
    path.push("methods");
    path.push("mod");
    path.set_extension("rs");
    fs::write(path, string).unwrap();
}

fn write_enums_mod<'a, P: AsRef<Path>>(dir: P, modules: impl Iterator<Item = &'a Module>) {
    let mut path = dir.as_ref().to_path_buf();
    let mut string = parse_all_mod(modules);
    let mut scope = Scope::new();
    scope.import("super", "*");
    string.push_str(&scope.to_string());
    path.push("types");
    path.push("enums");
    path.push("mod");
    path.set_extension("rs");
    fs::write(path, string).unwrap();
}

fn parse_all_mod<'a>(modules: impl Iterator<Item = &'a Module>) -> String {
    let mut string = String::new();
    for module in modules {
        parse_single_mod(module, &mut string);
    }
    string
}

fn parse_single_mod(module: &Module, string: &mut String) {
    let mut scope = Scope::new();
    scope.raw(&format!("mod {};", &module.module_name));
    string.push_str(&scope.to_string());
    string.push_str("\n");
    let mut scope = Scope::new();
    scope
        .import(
            &format!("self::{}", module.module_name),
            &module.module_type,
        )
        .vis("pub");
    string.push_str(&scope.to_string());
}
