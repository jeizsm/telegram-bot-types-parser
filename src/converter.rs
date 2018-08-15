use types::*;
use utils::capitalize;

impl From<TelegramMethod> for Type {
    fn from(method: TelegramMethod) -> Self {
        let fields = method.fields.into_iter().map(Into::into).collect();
        let mut name = method.name;
        capitalize(&mut name);
        let return_type = method.return_type.into();
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
        let field_type = field.telegram_type.into();
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

impl From<TelegramFieldType> for FieldType {
    fn from(field_type: TelegramFieldType) -> Self {
        let array_count = field_type.name.matches("Array of ").count();
        let mut type_name = field_type.name.replacen("Array of ", "", array_count);
        let contains_or = type_name.contains(" or ");
        let kind = if contains_or || type_name.contains(" and ") {
            let variants: Vec<_> = if contains_or {
                type_name.split(" or ")
            } else {
                type_name.split(" and ")
            }.map(ToOwned::to_owned)
                .collect();
            type_name = variants.join("Or");
            FieldKind::Enum(variants)
        } else {
            FieldKind::Simple
        };
        Self {
            is_boxed: false,
            array_count,
            doc: None,
            kind,
            name: type_name,
            is_optional: field_type.is_optional,
        }
    }
}
