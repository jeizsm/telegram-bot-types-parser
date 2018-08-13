extern crate codegen;
extern crate kuchiki;
mod converter;
mod generator;
mod parser;
mod types;
mod utils;
mod writer;

use generator::Generator;
use kuchiki::traits::TendrilSink;
use parser::{enum_parser, parser};
use std::collections::HashSet;
use std::{env, fs};
use types::{Type, TypeKind};
use writer::write_mod_files;

fn main() {
    let mut args = env::args().skip(1);
    let telegram_html_file = args.next().unwrap();
    let dir = args.next().unwrap();
    let html = fs::read_to_string(telegram_html_file).unwrap();
    let document = kuchiki::parse_html().one(html);

    let parsed = parser(&document);
    let converted: Vec<Type> = parsed.into_iter().map(Into::into).collect();
    let enum_parsed = enum_parser(&document);

    let mut modules = HashSet::new();
    for i in converted {
        i.generate(&mut modules);
    }

    for i in enum_parsed {
        i.generate(&mut modules);
    }

    let types = modules.iter().filter(|module| module.kind == TypeKind::Type);
    let methods = modules.iter().filter(|module| module.kind == TypeKind::Method);
    let enums = modules.iter().filter(|module| module.kind == TypeKind::Enum);
    write_mod_files(&dir, &TypeKind::Type, types);
    write_mod_files(&dir, &TypeKind::Method, methods);
    write_mod_files(&dir, &TypeKind::Enum, enums);
}
