use types::*;
use utils::{camel_case, capitalize};

impl From<TelegramMethod> for Type {
    fn from(method: TelegramMethod) -> Self {
        let fields = method.fields.into_iter().map(Into::into).collect();
        let mut name = method.name;
        capitalize(&mut name);
        let return_type = method.return_type;
        let field_name = return_type.replace(" ", "_");
        let return_type = TelegramFieldType {
            name: return_type,
            is_optional: false,
        };
        let return_type = return_type.into_field_type(&field_name);
        Self {
            name,
            docs: method.docs,
            fields,
            kind: TypeKind::Method(return_type),
        }
    }
}

impl From<TelegramType> for Type {
    fn from(telegram_type: TelegramType) -> Self {
        let fields = telegram_type.fields.into_iter().map(Into::into).collect();
        Self {
            name: telegram_type.name,
            docs: telegram_type.docs,
            fields,
            kind: TypeKind::Type,
        }
    }
}

impl From<TelegramField> for Field {
    fn from(field: TelegramField) -> Self {
        let field_type = field.telegram_type.into_field_type(&field.name);
        Self {
            doc: field.doc,
            name: field.name,
            field_type,
        }
    }
}

impl From<TelegramTypeOrMethod> for Type {
    fn from(method: TelegramTypeOrMethod) -> Self {
        match method {
            TelegramTypeOrMethod::Method(method) => method.into(),
            TelegramTypeOrMethod::Type(telegram_type) => telegram_type.into(),
        }
    }
}

impl TelegramFieldType {
    fn into_field_type(self, field_name: &str) -> FieldType {
        let array_count = self.name.matches("Array of ").count();
        let mut type_name = self.name.replacen("Array of ", "", array_count);
        let contains_or = type_name.contains(" or ");
        let kind = if contains_or || type_name.contains(" and ") {
            let variants = if contains_or {
                type_name.split(" or ")
            } else {
                type_name.split(" and ")
            }.map(ToOwned::to_owned)
                .collect();
            type_name = camel_case(field_name);
            FieldKind::Enum(variants)
        } else {
            FieldKind::Simple
        };
        FieldType {
            is_boxed: false,
            array_count,
            doc: None,
            kind,
            name: type_name,
            is_optional: self.is_optional,
        }
    }
}
