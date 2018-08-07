#[derive(Debug)]
pub struct TelegramMethod {
    pub name: String,
    pub docs: Vec<String>,
    pub fields: Vec<TelegramField>,
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
    pub optional: bool,
}

#[derive(Debug)]
pub enum TelegramTypeOrMethod {
    Type(TelegramType),
    Method(TelegramMethod),
}

#[derive(Debug)]
pub struct RustField {
    pub name: String,
    pub rust_type: RustType,
    pub doc: String,
}

#[derive(Debug)]
pub struct RustStruct {
    pub name: String,
    pub docs: Vec<String>,
    pub fields: Vec<RustField>,
    pub kind: Kind,
}

#[derive(Debug)]
pub enum RustType {
    String(String),
    Enum(RustEnum),
}

#[derive(Debug)]
pub struct RustEnum {
    pub is_array: bool,
    pub is_optional: bool,
    pub variants: Vec<String>,
    pub doc: Option<String>,
    pub name: String,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Kind {
    Type,
    Method,
    Enum,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Module {
    pub kind: Kind,
    pub module_name: String,
    pub module_type: String,
}
