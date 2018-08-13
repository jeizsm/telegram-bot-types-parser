use codegen::{Scope, Field as CodegenField};
use std::collections::HashSet;
use types::*;
use utils::*;

pub trait Generator {
    type ReturnType;

    fn generate(self, modules: &mut HashSet<Module>) -> Self::ReturnType;
}

impl Generator for Type {
    type ReturnType = String;

    fn generate(self, modules: &mut HashSet<Module>) -> Self::ReturnType {
        let mut scope = Scope::new();
        {
            scope.import("super", "*");
            let new_struct = scope
                .new_struct(&self.name)
                .doc(&self.docs.join("\n"))
                .derive("Serialize")
                .derive("Deserialize")
                .derive("Debug")
                .vis("pub");
            for field in self.fields {
                new_struct.push_field(field.generate(modules));
            }
        }
        let contents = scope.to_string();
        let module = Module {
            kind: self.kind,
            contents: contents,
            module_name: snake_case(&self.name),
            module_type: self.name,
        };
        modules.insert(module);
        scope.to_string()
    }
}

impl Generator for Field {
    type ReturnType = CodegenField;

    fn generate(mut self, modules: &mut HashSet<Module>) -> Self::ReturnType {
        let field_type = match self.name.as_ref() {
            "pinned_message" | "reply_to_message" => {
                self.field_type.is_boxed = true;
                self.field_type.generate(modules)
            }
            _ => self.field_type.generate(modules),
        };
        let mut field = match self.name.as_ref() {
            "type" => {
                let mut field = CodegenField::new("type_", field_type);
                field.annotation(vec![r#"#[serde(rename = "type")]"#]);
                field
            }
            name => CodegenField::new(name, field_type),
        };
        field.vis("pub").doc(&self.doc);
        field
    }
}

impl Generator for FieldType {
    type ReturnType = String;

    fn generate(self, modules: &mut HashSet<Module>) -> Self::ReturnType {
        if let FieldKind::Enum(variants) = self.kind {
            let mut scope = Scope::new();
            {
                scope.import("super", "*");
                let new_enum = scope
                    .new_enum(&self.name)
                    .derive("Serialize")
                    .derive("Deserialize")
                    .derive("Debug")
                    .vis("pub")
                    .annotation(vec![r#"serde(untagged)"#]);
                for string_variant in variants {
                    let variant = new_enum.new_variant(&string_variant);
                    variant.tuple(&string_variant);
                }
            }
            let contents = scope.to_string();
            let module = Module {
                kind: TypeKind::Enum,
                contents: contents,
                module_name: snake_case(&self.name),
                module_type: self.name.clone(),
            };
            modules.insert(module);
        };
        let mut field_type = self.name;
        field_type = match field_type.as_ref() {
            "Boolean" => "bool".to_string(),
            "Float number" => "Float".to_string(),
            _ => field_type,
        };
        if self.is_boxed {
            field_type = format!("Box<{}>", field_type);
        }
        if self.array_count != 0 {
            for _ in 0..self.array_count {
                field_type.insert_str(0, "Vec<");
                field_type.push_str(">");
            }
        }
        if self.is_optional {
            field_type = format!("Option<{}>", field_type);
        }
        field_type
    }
}

pub fn generate_single_mod(module: &Module, string: &mut String) {
    string.insert_str(0, &format!("mod {};\n", &module.module_name));
    let mut scope = Scope::new();
    scope
        .import(
            &format!("self::{}", module.module_name),
            &module.module_type,
        )
        .vis("pub");
    string.push_str(&scope.to_string());
}
