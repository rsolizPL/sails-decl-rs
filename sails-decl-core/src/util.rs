extern crate swc_common;
extern crate swc_ecma_parser;
use swc_common::sync::Lrc;
use swc_common::{FileName, SourceMap};
use swc_ecma_parser::{Parser, StringInput, Syntax, lexer::Lexer};
use swc_ecmascript::ast::PropName;
use swc_ecmascript::ast::TsType::{self};

pub fn get_prop_as_str(prop: &PropName) -> Option<&str> {
    match prop {
        PropName::Ident(ident) => Some(ident.sym.as_str()),
        PropName::Str(string) => string.value.as_str(),
        _ => None,
    }
}

pub fn parse_type_hint(type_hint: &str) -> Result<TsType, ()> {
    let temp_type = format!("0 as {}", type_hint);
    let cm: Lrc<SourceMap> = Default::default();
    let type_file = cm.new_source_file(FileName::Anon.into(), temp_type);

    let type_lexer = Lexer::new(
        Syntax::Typescript(Default::default()),
        Default::default(),
        StringInput::from(&*type_file),
        None,
    );

    let mut type_parser = Parser::new_from(type_lexer);

    if let Ok(expr) = type_parser.parse_expr() {
        if let Some(_ts_as) = expr.as_ts_as() {
            Ok(*_ts_as.type_ann.clone())
        } else {
            Err(())
        }
    } else {
        Err(())
    }
}
