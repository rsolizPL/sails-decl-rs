extern crate swc_common;
extern crate swc_ecma_parser;

use swc_common::sync::Lrc;
use swc_common::{
    errors::{ColorConfig, Handler},
    FileName, SourceMap,
};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};
use swc_ecmascript::ast::TsType::{self};
use swc_ecmascript::ast::{Expr, Ident, Lit, PropName, Str, TsInterfaceBody, TsKeywordType, TsKeywordTypeKind, TsPropertySignature, TsTypeAnn, TsTypeElement};
use swc_ecmascript::{
    ast::{TsInterfaceDecl},
};
#[derive(Debug)]
pub enum GenDeclForModelError {
    ParseError,
    InvalidModel,
}

pub fn gen_decl_for_model(code: String, model_name: String) -> Result<TsInterfaceDecl, GenDeclForModelError> {
    let cm: Lrc<SourceMap> = Default::default();
    let handler =
        Handler::with_tty_emitter(ColorConfig::Auto, true, false,
        Some(cm.clone()));

    let file = cm.new_source_file(FileName::Anon.into(), code);

    let lexer = Lexer::new(
        Syntax::Es(Default::default()),
        Default::default(),
        StringInput::from(&*file),
        None
    );

    let mut parser = Parser::new_from(lexer);

    for e in parser.take_errors() {
        e.into_diagnostic(&handler).emit();
    }

    let _module_res = parser.parse_commonjs().map_err(|e| {
            // Unrecoverable fatal error occurred
            e.into_diagnostic(&handler).emit()
        });
    
    if _module_res.is_err() {
        return Err(GenDeclForModelError::ParseError);
    }

    let _module = _module_res.unwrap();

    let mut elements: Vec<TsTypeElement> = vec![];

    let mut _found_valid_model = false;

    // look for module.exports, then attributes
    for _item in _module.body {
        if !_item.is_expr() {
            continue;
        }
        
        let _expression = _item.as_expr().unwrap();

        if !_expression.expr.is_assign() {
            continue;
        }

        let _assignment = _expression.expr.as_assign().unwrap();

        if !_assignment.left.is_simple() {
            continue;
        }

        let _simple_left = _assignment.left.as_simple().unwrap();

        if !_simple_left.is_member() {
            continue;
        }

        let _member_expr = _simple_left.as_member().unwrap();

        if !_member_expr.obj.is_ident() {
            continue;
        }

        let _obj_ident = _member_expr.obj.as_ident().unwrap();

        if _obj_ident.sym != *"module" {
            continue;
        }

        if !_member_expr.prop.is_ident() {
            continue;
        }

        let _prop_ident = _member_expr.prop.as_ident().unwrap();

        if _prop_ident.sym != *"exports" {
            continue;
        }

        _found_valid_model = true;

        // Now look for attributes inside the right hand side
        if !_assignment.right.is_object() {
            continue;
        }

        let _obj_lit = _assignment.right.as_object().unwrap();

        for _prop in &_obj_lit.props {
            // look for attributes property
            if !_prop.is_prop() {
                continue;
            }

            let _key_value = _prop.as_prop().unwrap();

            if !_key_value.is_key_value() {
                continue;
            }

            let _key_value_prop = _key_value.as_key_value().unwrap();

            if !_key_value_prop.key.is_ident() {
                continue;
            }

            let _key_ident = _key_value_prop.key.as_ident().unwrap();

            if _key_ident.sym != *"attributes" {
                continue;
            }

            // We found attributes, now we can process the properties
            if !_key_value_prop.value.is_object() {
                continue;
            }

            let _attributes_obj = _key_value_prop.value.as_object().unwrap();

            for _attribute_prop in &_attributes_obj.props {
                // Process each attribute property here
                // For simplicity, we will skip the actual processing logic
                // and just print the attribute names

                if !_attribute_prop.is_prop() {
                    continue;
                }

                let _attr_key_value = _attribute_prop.as_prop().unwrap();

                if !_attr_key_value.is_key_value() {
                    continue;
                }

                let _attr_key_value_prop = _attr_key_value.as_key_value().unwrap();

                if !_attr_key_value_prop.key.is_ident() {
                    continue;
                }

                let _attr_key_ident = _attr_key_value_prop.key.as_ident().unwrap();


                if !_attr_key_value_prop.value.is_object() {
                    continue;
                }

                let _attr_value_obj = _attr_key_value_prop.value.as_object().unwrap();

                let mut _type_prop_value: Option<&str> = None;
                let mut _type_hint_prop_value: Option<&str> = None;
                let mut _required_prop_value: bool = false;
                
                for _attr_field in &_attr_value_obj.props {
                    if !_attr_field.is_prop() {
                        continue;
                    }

                    let _attr_field_prop = _attr_field.as_prop().unwrap();
                    if !_attr_field_prop.is_key_value() {
                        continue;
                    }

                    let _attr_field_key_value = _attr_field_prop.as_key_value().unwrap();

                    let _attr_field_key_ident = match &_attr_field_key_value.key {
                        PropName::Ident(ident) => ident.sym.as_str(),
                        PropName::Str(string) => string.value.as_str().unwrap(),
                        _ => continue,
                    };

                    match _attr_field_key_ident {
                        "type" => {
                            if _attr_field_key_value.value.is_lit() {
                                let _lit = _attr_field_key_value.value.as_lit().unwrap();
                                if _lit.is_str() {
                                    _type_prop_value = _lit.as_str().unwrap().value.as_str();
                                }
                            }
                        }
                        "required" => {
                            if _attr_field_key_value.value.is_lit() {
                                let _lit = _attr_field_key_value.value.as_lit().unwrap();
                                if _lit.is_bool() {
                                    _required_prop_value = _lit.as_bool().unwrap().value;
                                }
                            }
                        }
                        "$SD-type-hint" => {
                            println!("Found $SD-type-hint");
                            if _attr_field_key_value.value.is_lit() {
                                let _lit = _attr_field_key_value.value.as_lit().unwrap();
                                if _lit.is_str() {
                                    _type_hint_prop_value = _lit.as_str().unwrap().value.as_str();
                                }
                            }
                        }
                        _ => {
                            continue;
                        }
                    }
                }

                let type_annotation: Result<TsTypeAnn, ()> = match _type_hint_prop_value.or(_type_prop_value) {
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
                    Some(x) => {
                        println!("Trying to parse type hint: {}", x);
                        // try parsing it, and see if you can make a type out of it
                        let temp_type = format!("0 as {}", x);
                        let type_file = cm.new_source_file(FileName::Anon.into(), temp_type);

                        let type_lexer = Lexer::new(
                            Syntax::Typescript(Default::default()),
                            Default::default(),
                            StringInput::from(&*type_file),
                            None
                        );

                        let mut type_parser = Parser::new_from(type_lexer);

                        if let Ok(expr) = type_parser.parse_expr() {
                            if let Some(_ts_as) = expr.as_ts_as() {
                                Ok(TsTypeAnn {
                                    span: Default::default(),
                                    type_ann: _ts_as.type_ann.clone(),
                                })
                            }else {
                                Err(())
                            }
                        }else {
                            Err(())
                        }
                    }
                    None => continue,
                };

                if _type_prop_value.is_some() {
                    elements.push(TsTypeElement::TsPropertySignature(TsPropertySignature{
                        span: Default::default(),
                        readonly: true,
                        key: Box::new(
                            Expr::Lit(Lit::Str(Str{
                                span: Default::default(),
                                value: _attr_key_ident.sym.as_str().into(),
                                raw: None 
                            }))
                        ),
                        computed: true,
                        optional: !_required_prop_value,
                        type_ann: Some(Box::new(type_annotation.unwrap())),
                    }));
                }

                
            }


        }
    }

    if !_found_valid_model {
        return Err(GenDeclForModelError::InvalidModel);
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
        }
    })
}