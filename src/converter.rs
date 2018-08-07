use types::*;
use utils::{camel_case, capitalize};

trait IntoRustType {
    fn into_type(self, name: &str) -> RustType;
}

trait IntoRustEnum {
    fn into_enum(self, name: &str) -> RustEnum;
}

trait IntoRustTypeString {
    fn into_string(self, name: &str) -> String;
}

impl From<TelegramMethod> for RustStruct {
    fn from(method: TelegramMethod) -> Self {
        let fields = method.fields.into_iter().map(|a| a.into()).collect();
        let mut name = method.name;
        capitalize(&mut name);
        Self {
            name,
            docs: method.docs,
            fields,
            kind: Kind::Method,
        }
    }
}

impl From<TelegramType> for RustStruct {
    fn from(telegram_type: TelegramType) -> Self {
        let fields = telegram_type.fields.into_iter().map(|a| a.into()).collect();
        Self {
            name: telegram_type.name,
            docs: telegram_type.docs,
            fields,
            kind: Kind::Type,
        }
    }
}

impl From<TelegramField> for RustField {
    fn from(field: TelegramField) -> Self {
        let rust_type = field.telegram_type.into_type(&field.name);
        Self {
            doc: field.doc,
            name: field.name,
            rust_type,
        }
    }
}

impl From<TelegramTypeOrMethod> for RustStruct {
    fn from(method: TelegramTypeOrMethod) -> Self {
        match method {
            TelegramTypeOrMethod::Method(method) => method.into(),
            TelegramTypeOrMethod::Type(telegram_type) => telegram_type.into(),
        }
    }
}

impl IntoRustType for TelegramFieldType {
    fn into_type(self, name: &str) -> RustType {
        if self.0.contains(" or ") || self.0.contains(" and ") {
            RustType::Enum(self.into_enum(name))
        } else {
            RustType::String(self.into_string(name))
        }
    }
}

impl IntoRustTypeString for TelegramFieldType {
    fn into_string(self, field_name: &str) -> String {
        let mut string = match (self.0.as_ref(), field_name) {
            ("Float number", _) => "Float".to_string(),
            ("Boolean", _) => "bool".to_string(),
            (_, "pinned_message") => "Box<Message>".to_string(),
            (_, "reply_to_message") => "Box<Message>".to_string(),
            _ => self.0,
        };
        loop {
            let start_index = string.find("Array of ");
            match start_index {
                Some(start_index) => {
                    let end_index = start_index + "Array of ".len();
                    string.replace_range(start_index..end_index, "Vec<");
                    string.push_str(">");
                }
                None => break,
            }
        }
        match self.1 {
            true => format!("Option<{}>", string),
            false => string,
        }
    }
}

impl IntoRustEnum for TelegramFieldType {
    fn into_enum(self, name: &str) -> RustEnum {
        let is_array = self.0.starts_with("Array of ");
        let variants = if is_array {
            self
                .0
                .replacen("Array of ", "", 1)
                .split(" and ")
                .map(|string| string.to_string())
                .collect()
        } else {
            self
                .0
                .split(" or ")
                .map(|string| string.to_string())
                .collect()
        };
        RustEnum {
            is_optional: self.1,
            is_array: is_array,
            variants,
            doc: None,
            name: camel_case(&name)
        }
    }
}
