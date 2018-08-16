#[derive(Debug)]
pub struct TelegramMethod {
    pub name: String,
    pub docs: Vec<String>,
    pub fields: Vec<TelegramField>,
    pub return_type: TelegramFieldType,
}

#[derive(Debug)]
pub struct TelegramType {
    pub name: String,
    pub docs: Vec<String>,
    pub fields: Vec<TelegramField>,
}

#[derive(Debug)]
pub struct TelegramField {
    pub name: String,
    pub doc: String,
    pub telegram_type: TelegramFieldType,
}

#[derive(Debug)]
pub struct TelegramFieldType {
    pub name: String,
    pub is_optional: bool,
}

#[derive(Debug)]
pub enum TelegramTypeOrMethod {
    Type(TelegramType),
    Method(TelegramMethod),
}

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
    pub doc: String,
}

#[derive(Debug)]
pub struct Type {
    pub name: String,
    pub docs: Vec<String>,
    pub fields: Vec<Field>,
    pub kind: TypeKind,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct FieldType {
    pub array_count: usize,
    pub is_optional: bool,
    pub doc: Option<String>,
    pub name: String,
    pub kind: FieldKind,
    pub is_boxed: bool,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum TypeKind {
    Type,
    Method(FieldType),
    Enum,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum FieldKind {
    Simple,
    Enum(Vec<(String, String)>),
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Module {
    pub kind: TypeKind,
    pub module_name: String,
    pub module_type: String,
    pub contents: String,
}
