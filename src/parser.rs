use kuchiki::{
    iter::NodeIterator,
    {ElementData, NodeDataRef, NodeRef},
};
use types::*;

trait Parse {
    fn parse(node: &NodeRef) -> Self;

    fn parse_name_and_docs(table_siblings: impl Iterator<Item = NodeRef>) -> (String, Vec<String>) {
        let mut docs = Vec::new();
        let nodes = table_siblings.skip(1).step_by(2);
        for node in nodes {
            let element = node.as_element().unwrap();
            let name = element.name.local.to_string();
            match name.as_str() {
                "h4" => return (Self::parse_name(&node), docs),
                _ => docs.insert(0, Self::parse_doc(&node)),
            }
        }
        panic!("cannot parse name and docs");
    }

    fn parse_name(node: &NodeRef) -> String {
        let child = node.last_child().unwrap();
        match child.as_text() {
            Some(text) => text.borrow().to_owned(),
            None => panic!("cannot parse name"),
        }
    }

    fn parse_doc(node: &NodeRef) -> String {
        let nodes = node.children();
        nodes
            .map(|node| match node.as_text() {
                Some(text) => text.borrow().to_owned(),
                None => Self::parse_doc(&node),
            })
            .collect()
    }

    fn parse_fields(_: impl Iterator<Item = NodeDataRef<ElementData>>) -> Vec<TelegramField> {
        unimplemented!()
    }

    fn parse_field(node: &NodeRef) -> String {
        let child = node.first_child().unwrap();
        match child.as_text() {
            Some(text) => text.borrow().to_owned(),
            None => panic!("something wrong"),
        }
    }

    fn parse_type(node: &NodeRef) -> String {
        node.children()
            .map(|child| match child.as_text() {
                Some(text) => text.borrow().to_owned(),
                None => Self::parse_type(&child),
            })
            .collect()
    }

    fn parse_required(_: &NodeRef) -> String {
        unimplemented!()
    }
}

impl Parse for TelegramTypeOrMethod {
    fn parse(table: &NodeRef) -> Self {
        let mut trs = table.select("tbody > tr").unwrap();
        let header = trs.next().unwrap();
        let header = header.as_node();
        let count = header.children().select("td").unwrap().count();
        match count {
            3 => TelegramTypeOrMethod::Type(TelegramType::parse(table)),
            4 => TelegramTypeOrMethod::Method(TelegramMethod::parse(table)),
            _ => panic!("not type or method"),
        }
    }
}

impl Parse for TelegramType {
    fn parse(table: &NodeRef) -> Self {
        let (name, docs) = Self::parse_name_and_docs(table.preceding_siblings());
        let fields = Self::parse_fields(table.select("tr").unwrap());
        Self { name, docs, fields }
    }

    fn parse_fields(trs: impl Iterator<Item = NodeDataRef<ElementData>>) -> Vec<TelegramField> {
        trs.skip(1)
            .map(|tr| {
                let tr = tr.as_node();
                let tds = tr.children().select("td").unwrap().enumerate();
                let field: Vec<String> =
                    tds.map(|(i, td)| match i {
                        0 => Self::parse_field(&td.as_node()),
                        1 => Self::parse_type(&td.as_node()),
                        2 => Self::parse_doc(&td.as_node()),
                        _ => panic!("no field, type, doc or required field"),
                    }).collect();
                let doc = &field[2];
                let is_optional = doc.starts_with("Optional. ");
                let doc = if is_optional {
                    doc.replace("Optional. ", "")
                } else {
                    doc.to_owned()
                };
                TelegramField {
                    name: field[0].clone(),
                    telegram_type: TelegramFieldType {
                        name: field[1].clone(),
                        is_optional,
                    },
                    doc,
                }
            })
            .collect()
    }
}

impl Parse for TelegramMethod {
    fn parse(table: &NodeRef) -> Self {
        let (name, docs) = Self::parse_name_and_docs(table.preceding_siblings());
        let fields = Self::parse_fields(table.select("tr").unwrap());
        let return_type = Self::parse_return_type(&docs[0]);
        Self {
            name,
            docs,
            fields,
            return_type,
        }
    }

    fn parse_fields(trs: impl Iterator<Item = NodeDataRef<ElementData>>) -> Vec<TelegramField> {
        trs.skip(1)
            .map(|tr| {
                let tr = tr.as_node();
                let tds = tr.children().select("td").unwrap().enumerate();
                let field: Vec<String> =
                    tds.map(|(i, td)| match i {
                        0 => Self::parse_field(&td.as_node()),
                        1 => Self::parse_type(&td.as_node()),
                        2 => Self::parse_required(&td.as_node()),
                        3 => Self::parse_doc(&td.as_node()),
                        _ => panic!("no field, type, doc or required field"),
                    }).collect();
                TelegramField {
                    name: field[0].clone(),
                    telegram_type: TelegramFieldType {
                        name: field[1].clone(),
                        is_optional: "Optional" == field[2],
                    },
                    doc: field[3].clone(),
                }
            })
            .collect()
    }

    fn parse_required(node: &NodeRef) -> String {
        let child = node.first_child().unwrap();
        match child.as_text() {
            Some(text) => text.borrow().to_string(),
            None => panic!("is not required field"),
        }
    }
}

impl TelegramMethod {
    fn parse_return_type(doc: &str) -> String {
        let sentences = doc.split(".");
        for sentence in sentences {
            if sentence.contains("is returned") || sentence.contains("eturns") {
                if sentence.contains("otherwise") {
                    return Self::parse_return_type_sentence(sentence, 2);
                } else {
                    return Self::parse_return_type_sentence(sentence, 1);
                }
            }
        }
        unreachable!()
    }

    fn parse_return_type_sentence(sentence: &str, count: u8) -> String {
        let mut string = String::new();
        if sentence.contains("rray of") {
            string.push_str("Array of ")
        }
        let words: Vec<_> = sentence.split_whitespace().collect();
        let mut iterator = words.iter().enumerate();
        for i in 0..count {
            if i != 0 {
                string.push_str(" or ")
            };
            let (position, word) = iterator
                .find(|&(_index, word)| word.starts_with("returned") || word.ends_with("eturns"))
                .unwrap();
            match (position, *word) {
                (position, "returned") | (position, "returned,") => {
                    string.push_str(Self::parse_is_returned(&words, position))
                }
                (position, "Returns") | (position, "returns") => {
                    string.push_str(Self::parse_returns(&words, position))
                }
                (_, _) => unreachable!(),
            }
        }
        string
    }

    fn parse_returns<'a>(words: &[&'a str], position: usize) -> &'a str {
        let mut word = if let Some(position) = words.iter().position(|&word| word == "as") {
            words[position + 1]
        } else if let Some(position) = words.iter().position(|&word| word == "Array") {
            words[position + 2]
        } else {
            words[position + 1]
        };
        if word == "a" {
            word = words[position + 2]
        } else if word == "the" {
            word = words[position + 3]
        }
        if word.ends_with(",") {
            word = &word[0..word.len() - 1]
        }
        word
    }

    fn parse_is_returned<'a>(words: &[&'a str], position: usize) -> &'a str {
        let mut word = words[position - 2];
        if word.starts_with("object") {
            word = words[position - 3]
        }
        if word.ends_with("s") {
            word = &word[0..word.len() - 1]
        }
        word
    }
}

impl Parse for FieldType {
    fn parse(node: &NodeRef) -> Self {
        let mut siblings = node.preceding_siblings();
        let p = siblings.nth(1).unwrap();
        let h4 = siblings.nth(1).unwrap();
        let doc = Self::parse_doc(&p);
        let name = Self::parse_name(&h4);
        let variants = node
            .select("li")
            .unwrap()
            .map(|li| {
                li.as_node()
                    .first_child()
                    .unwrap()
                    .first_child()
                    .unwrap()
                    .to_string()
            })
            .collect();
        Self {
            name,
            doc: Some(doc),
            array_count: 0,
            is_optional: false,
            kind: FieldKind::Enum(variants),
            is_boxed: false,
        }
    }
}

pub fn parser(document: &NodeRef) -> impl Iterator<Item = TelegramTypeOrMethod> {
    let css_selector = "h4 + p ~ table";
    document.select(css_selector).unwrap().map(|table| {
        let table = table.as_node();
        TelegramTypeOrMethod::parse(table)
    })
}

pub fn enum_parser(document: &NodeRef) -> impl Iterator<Item = FieldType> {
    let css_selector = "h4 + p + ul";
    document.select(css_selector).unwrap().skip(2).map(|ul| {
        let ul = ul.as_node();
        FieldType::parse(ul)
    })
}
