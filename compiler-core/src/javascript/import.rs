use std::collections::{HashMap, HashSet};

use itertools::Itertools;

use crate::{
    docvec,
    javascript::{JavaScriptCodegenTarget, INDENT},
    pretty::{break_str, break_, concat, join, line, Document, Documentable},
};

/// A collection of JavaScript import statements from Gleam imports and from
/// external functions, to be rendered into a JavaScript module.
///
#[derive(Debug, Default)]
pub(crate) struct Imports {
    imports: HashMap<String, Import>,
    exports: HashSet<String>,
}

impl Imports {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_export(&mut self, export: String) {
        let _ = self.exports.insert(export);
    }

    pub fn register_module(
        &mut self,
        path: String,
        aliases: impl IntoIterator<Item = String>,
        unqualified_imports: impl IntoIterator<Item = Member>,
    ) {
        let import = self
            .imports
            .entry(path.clone())
            .or_insert_with(|| Import::new(path.clone()));
        import.aliases.extend(aliases);
        import.unqualified.extend(unqualified_imports)
    }

    pub fn into_doc(self, codegen_target: JavaScriptCodegenTarget) -> Document {
        let imports = concat(
            self.imports
                .into_values()
                .sorted_by(|a, b| a.path.cmp(&b.path))
                .map(|import| Import::into_doc(import, codegen_target)),
        );

        if self.exports.is_empty() {
            imports
        } else {
            let names = join(
                self.exports.into_iter().sorted().map(Document::String),
                break_str(",", ", "),
            );
            let names = docvec![
                docvec![break_str("", " "), names].nest(INDENT),
                break_str(",", " ")
            ]
            .group();
            imports
                .append(line())
                .append("export {")
                .append(names)
                .append("};")
                .append(line())
        }
    }

    pub fn is_empty(&self) -> bool {
        self.imports.is_empty() && self.exports.is_empty()
    }
}

#[derive(Debug)]
struct Import {
    path: String,
    aliases: HashSet<String>,
    unqualified: Vec<Member>,
}

impl Import {
    fn new(path: String) -> Self {
        Self {
            path,
            aliases: Default::default(),
            unqualified: Default::default(),
        }
    }

    pub fn into_doc(self, codegen_target: JavaScriptCodegenTarget) -> Document {
        let path = Document::String(self.path.clone());
        let import_modifier = if codegen_target == JavaScriptCodegenTarget::TypeScriptDeclarations {
            "type "
        } else {
            ""
        };
        let alias_imports = concat(self.aliases.into_iter().sorted().map(|alias| {
            docvec![
                "import ",
                import_modifier,
                "* as ",
                Document::String(alias),
                " from \"",
                path.clone(),
                r#"";"#,
                line()
            ]
        }));
        if self.unqualified.is_empty() {
            alias_imports
        } else {
            let members = self.unqualified.into_iter().map(Member::into_doc);
            let members = join(members, break_str(",", ", "));
            let members = docvec![
                docvec![break_str("", " "), members].nest(INDENT),
                break_str(",", " ")
            ]
            .group();
            docvec![
                alias_imports,
                "import ",
                import_modifier,
                "{",
                members,
                "} from \"",
                path,
                r#"";"#,
                line()
            ]
        }
    }
}

#[derive(Debug)]
pub struct Member {
    pub name: Document,
    pub alias: Option<Document>,
}

impl Member {
    fn into_doc(self) -> Document {
        match self.alias {
            None => self.name,
            Some(alias) => docvec![self.name, " as ", alias],
        }
    }
}

#[test]
fn into_doc() {
    let mut imports = Imports::new();
    imports.register_module("./gleam/empty".into(), [], []);
    imports.register_module(
        "./multiple/times".into(),
        ["wibble".into(), "wobble".into()],
        [],
    );
    imports.register_module("./multiple/times".into(), ["wubble".into()], []);
    imports.register_module(
        "./multiple/times".into(),
        [],
        [Member {
            name: "one".to_doc(),
            alias: None,
        }],
    );

    imports.register_module(
        "./other".into(),
        [],
        [
            Member {
                name: "one".to_doc(),
                alias: None,
            },
            Member {
                name: "one".to_doc(),
                alias: Some("onee".to_doc()),
            },
            Member {
                name: "two".to_doc(),
                alias: Some("twoo".to_doc()),
            },
        ],
    );

    imports.register_module(
        "./other".into(),
        [],
        [
            Member {
                name: "three".to_doc(),
                alias: None,
            },
            Member {
                name: "four".to_doc(),
                alias: None,
            },
        ],
    );

    imports.register_module(
        "./zzz".into(),
        [],
        [
            Member {
                name: "one".to_doc(),
                alias: None,
            },
            Member {
                name: "two".to_doc(),
                alias: None,
            },
        ],
    );

    assert_eq!(
        line()
            .append(imports.into_doc(JavaScriptCodegenTarget::JavaScript))
            .to_pretty_string(40),
        r#"
import * as wibble from "./multiple/times";
import * as wobble from "./multiple/times";
import * as wubble from "./multiple/times";
import { one } from "./multiple/times";
import {
  one,
  one as onee,
  two as twoo,
  three,
  four,
} from "./other";
import { one, two } from "./zzz";
"#
        .to_string()
    );
}
