extern crate codegen;
extern crate kuchiki;
mod converter;
mod parser;
mod types;
mod utils;
mod writer;

use kuchiki::traits::TendrilSink;
use parser::{enum_parser, parser};
use std::collections::HashSet;
use std::{env, fs};
use types::Kind;
use types::RustStruct;
use writer::{write_mod_files, Writer};

fn main() {
    let mut args = env::args().skip(1);
    let telegram_html_file = args.next().unwrap();
    let dir = args.next().unwrap();
    let html = fs::read_to_string(telegram_html_file).unwrap();
    let document = kuchiki::parse_html().one(html);

    let parsed = parser(&document);
    let converted: Vec<RustStruct> = parsed.into_iter().map(Into::into).collect();
    let enum_parsed = enum_parser(&document);

    let mut modules = HashSet::new();
    for i in converted {
        i.write(&dir, &mut modules);
    }

    for i in enum_parsed {
        i.write(&dir, &mut modules);
    }

    let types = modules.iter().filter(|module| module.kind == Kind::Type);
    let methods = modules.iter().filter(|module| module.kind == Kind::Method);
    let enums = modules.iter().filter(|module| module.kind == Kind::Enum);
    write_mod_files(&dir, &Kind::Type, types);
    write_mod_files(&dir, &Kind::Method, methods);
    write_mod_files(&dir, &Kind::Enum, enums);
}
