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
    let converted: Vec<_> = parsed.map(Into::<Type>::into).collect();
    let mut return_types: HashSet<_> = converted.iter().filter_map(|ty| {
        if let TypeKind::Method(field) = &ty.kind {
            Some(field.name.clone())
        } else {
            None
        }
    }).collect();
    converted.iter().for_each(|ty| {
        if return_types.get(&ty.name).is_some() {
            ty.fields.iter().for_each(|field| {
                return_types.insert(field.field_type.name.clone());
            });
        };
    });
    let enum_parsed = enum_parser(&document);

    let mut modules = HashSet::new();
    for i in converted {
        i.generate(&mut modules, &return_types);
    }

    for i in enum_parsed {
        i.generate(&mut modules, &return_types);
    }

    let types = modules
        .iter()
        .filter(|module| module.kind == TypeKind::Type);
    let methods = modules.iter().filter(|module| {
        if let TypeKind::Method(_) = module.kind {
            true
        } else {
            false
        }
    });
    let enums = modules
        .iter()
        .filter(|module| module.kind == TypeKind::Enum);
    write_mod_files(&dir, types.peekable());
    write_mod_files(&dir, methods.peekable());
    write_mod_files(&dir, enums.peekable());
}
