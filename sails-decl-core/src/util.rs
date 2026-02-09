extern crate swc_common;
extern crate swc_ecma_parser;
use swc_common::sync::Lrc;
use swc_common::{FileName, SourceMap};
use swc_ecma_parser::{Parser, StringInput, Syntax, lexer::Lexer};
use swc_ecmascript::ast::TsType::{self};
use swc_ecmascript::ast::{
    ObjectLit, PropName, Script, TsKeywordType, TsKeywordTypeKind, TsUnionOrIntersectionType, TsUnionType
};

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

pub struct AttributeTypeInfo {
  pub ts_type: TsType,
  pub required: bool,
}

pub fn ts_type_from_attribute(attribute: &ObjectLit) -> Option<AttributeTypeInfo> {
    let mut attribute_type: Option<&str> = None;
    let mut attribute_type_hint: Option<&str> = None;
    let mut attribute_required: bool = false;
    let mut allows_null: bool = false;

    for _attr_field in &attribute.props {
        let _attr_field_prop = match _attr_field.as_prop() {
            Some(prop) => prop,
            None => continue,
        };

        let attribute_pair = _attr_field_prop.as_key_value().unwrap();

        let attribute_name = match get_prop_as_str(&attribute_pair.key) {
            Some(name) => name,
            None => continue,
        };
        let attribute_value = &attribute_pair.value;

        match attribute_name {
            "type" => {
                if attribute_value.is_lit() {
                    let _lit = attribute_value.as_lit().unwrap();
                    if _lit.is_str() {
                        attribute_type = _lit.as_str().unwrap().value.as_str();
                    }
                }
            }
            "required" => {
                if attribute_value.is_lit() {
                    let _lit = attribute_value.as_lit().unwrap();
                    if _lit.is_bool() {
                        attribute_required = _lit.as_bool().unwrap().value;
                    }
                }
            }
            "allowNull" => {
                if attribute_value.is_lit() {
                    let _lit = attribute_value.as_lit().unwrap();
                    if _lit.is_bool() {
                        allows_null = _lit.as_bool().unwrap().value;
                    }
                }
            }
            "$SD-type-hint" => {
                if attribute_value.is_lit() {
                    let _lit = attribute_value.as_lit().unwrap();
                    if _lit.is_str() {
                        attribute_type_hint = _lit.as_str().unwrap().value.as_str();
                    }
                }
            }
            _ => {
                continue;
            }
        }
    }

    match attribute_type_hint.or(attribute_type) {
        Some("string") => Some(TsType::TsKeywordType(TsKeywordType {
            span: Default::default(),
            kind: TsKeywordTypeKind::TsStringKeyword,
        })),
        Some("number") => Some(TsType::TsKeywordType(TsKeywordType {
            span: Default::default(),
            kind: TsKeywordTypeKind::TsNumberKeyword,
        })),
        Some("boolean") => Some(TsType::TsKeywordType(TsKeywordType {
            span: Default::default(),
            kind: TsKeywordTypeKind::TsBooleanKeyword,
        })),
        Some("json") => Some(TsType::TsKeywordType(TsKeywordType {
            span: Default::default(),
            kind: TsKeywordTypeKind::TsAnyKeyword,
        })),
        Some("ref") => Some(TsType::TsKeywordType(TsKeywordType {
            span: Default::default(),
            kind: TsKeywordTypeKind::TsAnyKeyword,
        })),
        Some(x) => parse_type_hint(x).ok(),
        None => None,
    }
    .map(|hint| AttributeTypeInfo {
        ts_type: if allows_null {
          hint
        } else {
          TsType::TsUnionOrIntersectionType(TsUnionOrIntersectionType::TsUnionType(TsUnionType {
            span: Default::default(),
            types: vec![
              Box::new(hint),
              Box::new(TsType::TsKeywordType(TsKeywordType {
                span: Default::default(),
                kind: TsKeywordTypeKind::TsNullKeyword,
              })),
            ],
          }))
        },
        required: attribute_required,
    })
}

pub fn find_module_exports(module: Script) -> Option<ObjectLit> {
    module
    .body
    .iter()
    .find_map(|item| {
        let assign = item.as_expr()?.expr.as_assign()?;
        let member = assign.left.as_simple()?.as_member()?;

        if member.obj.as_ident()?.sym != "module" || member.prop.as_ident()?.sym != "exports" {
            return None;
        }

        assign.right.as_object().cloned()
    })
}

pub struct EmittedCode {
    pub code: String,
    pub source_map: String,
}