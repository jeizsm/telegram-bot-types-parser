use codegen::{Field as CodegenField, Scope};
use std::collections::HashSet;
use types::*;
use utils::*;

pub trait Generator {
    type ReturnType;

    fn generate(self, modules: &mut HashSet<Module>, return_types: &HashSet<String>) -> Self::ReturnType;
}

impl Generator for Type {
    type ReturnType = String;

    fn generate(self, modules: &mut HashSet<Module>, return_types: &HashSet<String>) -> Self::ReturnType {
        let mut scope = Scope::new();
        scope.import("crate::types", "*");
        if let TypeKind::Method(return_type) = self.kind.clone() {
            let return_type = return_type.generate(modules, return_types);
            let return_type_annotation = format!(r#"return_type = "{}""#, return_type);
            let new_annotation = r#"new(vis = "pub")"#;
            let set_annotation = r#"set(vis = "pub")"#;
            let new_struct = scope
                .new_struct(&self.name)
                .doc(&self.docs.join("\n"))
                .derive("Debug")
                .derive("Serialize")
                .derive("TelegramApi")
                .derive("Setters")
                .derive("New")
                .annotation(vec![&return_type_annotation, new_annotation, set_annotation])
                .vis("pub");
            for field in self.fields {
                new_struct.push_field(field.generate(modules, return_types));
            }
        } else {
            let set_annotation = r#"set(vis = "pub")"#;
            let get_annotation = r#"get(vis = "pub")"#;
            let new_annotation = r#"new(vis = "pub")"#;
            let new_struct = scope
                .new_struct(&self.name)
                .doc(&self.docs.join("\n"))
                .derive("Debug")
                .vis("pub");
            {
                let return_type = return_types.get(&self.name).is_some();
                if return_type || self.name == "WebhookInfo" {
                    new_struct.derive("Deserialize").derive("Clone").derive("Getters");
                    new_struct.annotation(vec![get_annotation]);
                } else {
                    new_struct.derive("Serialize").derive("Setters").derive("New");
                    new_struct.annotation(vec![new_annotation, set_annotation]);
                }
                if self.name == "MaskPosition" {
                    new_struct.derive("Serialize").derive("Setters").derive("New");
                    new_struct.push_annotation(new_annotation).push_annotation(set_annotation);
                }
            }
            for field in self.fields {
                new_struct.push_field(field.generate(modules, return_types));
            }
        }
        let contents = scope.to_string();
        let module = Module {
            kind: self.kind,
            contents,
            module_name: snake_case(&self.name),
            module_type: self.name,
        };
        modules.insert(module);
        scope.to_string()
    }
}

impl Generator for Field {
    type ReturnType = CodegenField;

    fn generate(mut self, modules: &mut HashSet<Module>, return_types: &HashSet<String>) -> Self::ReturnType {
        let is_optional = self.field_type.is_optional;
        let field_type = match self.name.as_ref() {
            "pinned_message" | "reply_to_message" => {
                self.field_type.is_boxed = true;
                self.field_type.generate(modules, return_types)
            }
            _ => self.field_type.generate(modules, return_types),
        };
        let field_name = match self.name.as_ref() {
            "type" => "type_",
            name => name,
        };
        let mut field = CodegenField::new(field_name, &field_type);
        if field_name == "type_" {
            field.push_annotation(r#"serde(rename = "type")"#);
        }
        if is_optional {
            field.push_annotation(r#"serde(skip_serializing_if = "Option::is_none")"#);
        }
        field.doc(&self.doc);
        field.vis("pub(crate)");
        field
    }
}

impl Generator for FieldType {
    type ReturnType = String;

    fn generate(self, modules: &mut HashSet<Module>, return_types: &HashSet<String>) -> Self::ReturnType {
        if let FieldKind::Enum(variants) = self.kind {
            let mut scope = Scope::new();
            {
                scope.import("crate::types", "*");
                let new_enum = scope
                    .new_enum(&self.name)
                    .derive("Debug")
                    .vis("pub")
                    .annotation(vec![r#"serde(untagged)"#]);
                {
                    let return_type = return_types.get(&self.name).is_some();
                    if return_type {
                        new_enum.derive("Deserialize").derive("Clone");
                    } else {
                        new_enum.derive("Serialize");
                    }
                }
                for (variant_name, variant_type) in variants {
                    let variant = new_enum.new_variant(&variant_name);
                    variant.tuple(&variant_type);
                }
            }
            let contents = scope.to_string();
            let module = Module {
                kind: TypeKind::Enum,
                contents,
                module_name: snake_case(&self.name),
                module_type: self.name.clone(),
            };
            modules.insert(module);
        };
        let mut field_type = self.name;
        field_type = match field_type.as_ref() {
            "Boolean" => "bool".to_string(),
            "Float number" => "Float".to_string(),
            "Int" => "Integer".to_string(),
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
