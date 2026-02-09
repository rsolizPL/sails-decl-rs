extern crate swc_common;
extern crate swc_ecma_parser;

use swc_common::sync::Lrc;
use swc_common::{
    FileName, SourceMap,
    errors::{ColorConfig, Handler},
};
use swc_ecma_parser::{Parser, StringInput, Syntax, lexer::Lexer};
use swc_ecmascript::ast::TsInterfaceDecl;
use swc_ecmascript::ast::TsType::{self};
use swc_ecmascript::ast::{
    Expr, Ident, Lit, Str, TsInterfaceBody, TsKeywordType, TsKeywordTypeKind,
    TsPropertySignature, TsTypeAnn, TsTypeElement,
};

use crate::util::{get_prop_as_str, parse_type_hint};
#[derive(Debug)]
pub enum GenDeclarationsError {
    ParseError,
    InvalidModel,
    IsNotCommonJsModule,
    CommonJsModuleDoesNotExportObject,
    SDTypeHintParseError,
}

pub fn gen_decl(
    code: String,
    model_name: String,
) -> Result<TsInterfaceDecl, GenDeclarationsError> {
    let cm: Lrc<SourceMap> = Default::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

    let file = cm.new_source_file(FileName::Anon.into(), code);

    let lexer = Lexer::new(
        Syntax::Es(Default::default()),
        Default::default(),
        StringInput::from(&*file),
        None,
    );

    let mut parser = Parser::new_from(lexer);

    for e in parser.take_errors() {
        e.into_diagnostic(&handler).emit();
    }

    let module_result = parser.parse_commonjs().map_err(|e| {
        // Unrecoverable fatal error occurred
        e.into_diagnostic(&handler).emit()
    });

    if module_result.is_err() {
        return Err(GenDeclarationsError::ParseError);
    }

    let module = module_result.unwrap();

    let mut elements: Vec<TsTypeElement> = vec![];

    let module_exports_obj = module.body.iter().find_map(|item| {
        let assign = item.as_expr()?.expr.as_assign()?;
        let member = assign.left.as_simple()?.as_member()?;
        
        if member.obj.as_ident()?.sym != "module" || member.prop.as_ident()?.sym != "exports" {
            return None;
        }

        assign.right.as_object().cloned()
    }).ok_or(GenDeclarationsError::IsNotCommonJsModule)?;

    let attributes_obj = module_exports_obj.props.iter().find_map(|prop| {
        let _key_value_prop = prop.as_prop().and_then(|p| p.as_key_value())?;
        let key_name = get_prop_as_str(&_key_value_prop.key)?;

        if key_name != "attributes" {
            return None;
        }

        _key_value_prop.value.as_object().cloned()
    }).ok_or(GenDeclarationsError::InvalidModel)?;

    for attribute in &attributes_obj.props {
        let attribute_pair = match attribute.as_prop().and_then(|p| p.as_key_value()) {
            Some(prop) => prop,
            None => continue,
        };

        let _attr_key_ident = match attribute_pair.key.as_ident() {
            Some(ident) => ident,
            None => continue,
        };

        let _attr_value_obj = match attribute_pair.value.as_object() {
            Some(obj) => obj,
            None => continue,
        };

        let mut attribute_type: Option<&str> = None;
        let mut attribute_type_hint: Option<&str> = None;
        let mut attribute_required: bool = false;

        for _attr_field in &_attr_value_obj.props {
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

        let type_annotation: Result<TsTypeAnn, ()> = match attribute_type_hint.or(attribute_type) {
            Some("string") => Ok(TsTypeAnn {
                span: Default::default(),
                type_ann: Box::new(TsType::TsKeywordType(TsKeywordType {
                    span: Default::default(),
                    kind: TsKeywordTypeKind::TsStringKeyword,
                })),
            }),
            Some("number") => Ok(TsTypeAnn {
                span: Default::default(),
                type_ann: Box::new(TsType::TsKeywordType(TsKeywordType {
                    span: Default::default(),
                    kind: TsKeywordTypeKind::TsNumberKeyword,
                })),
            }),
            Some("boolean") => Ok(TsTypeAnn {
                span: Default::default(),
                type_ann: Box::new(TsType::TsKeywordType(TsKeywordType {
                    span: Default::default(),
                    kind: TsKeywordTypeKind::TsBooleanKeyword,
                })),
            }),
            Some("json") => Ok(TsTypeAnn {
                span: Default::default(),
                type_ann: Box::new(TsType::TsKeywordType(TsKeywordType {
                    span: Default::default(),
                    kind: TsKeywordTypeKind::TsAnyKeyword,
                })),
            }),
            Some(x) => parse_type_hint(x).map(|ty| TsTypeAnn {
                span: Default::default(),
                type_ann: Box::new(ty),
            }),
            None => continue,
        };

        if type_annotation.is_err() {
            return Err(GenDeclarationsError::SDTypeHintParseError);
        }

        if attribute_type.is_some() {
            elements.push(TsTypeElement::TsPropertySignature(TsPropertySignature {
                span: Default::default(),
                readonly: false,
                key: Box::new(Expr::Lit(Lit::Str(Str {
                    span: Default::default(),
                    value: _attr_key_ident.sym.as_str().into(),
                    raw: None,
                }))),
                computed: true,
                optional: !attribute_required,
                type_ann: Some(Box::new(type_annotation.unwrap())),
            }));
        }
    }

    Ok(TsInterfaceDecl {
        span: Default::default(),
        id: Ident {
            span: Default::default(),
            ctxt: Default::default(),
            sym: format!("{}__ModelDecl", model_name).into(),
            optional: false,
        },
        declare: true,
        type_params: None,
        extends: vec![],
        body: TsInterfaceBody {
            span: Default::default(),
            body: elements,
        },
    })
}
