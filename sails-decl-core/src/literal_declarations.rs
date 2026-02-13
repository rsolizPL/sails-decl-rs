use swc_ecmascript::ast::{
    BindingIdent, Decl, ExportDecl, Ident, ModuleItem, Pat, TsArrayType, TsFnParam, TsInterfaceBody, TsInterfaceDecl, TsKeywordType, TsKeywordTypeKind, TsMethodSignature, TsModuleBlock, TsModuleDecl, TsNamespaceBody, TsPropertySignature, TsType, TsTypeAnn, TsTypeElement, TsTypeParam, TsTypeParamDecl, TsTypeParamInstantiation, TsTypeRef, TsUnionOrIntersectionType, TsUnionType, VarDecl, VarDeclKind, VarDeclarator
};

fn as_ident(sym: &str) -> Ident {
    Ident {
        span: Default::default(),
        ctxt: Default::default(),
        sym: sym.into(),
        optional: false,
    }
}

pub fn import_named(module_name: &str, imported_items: Vec<&str>, type_only: bool) -> ModuleItem {
    ModuleItem::ModuleDecl(swc_ecmascript::ast::ModuleDecl::Import(swc_ecmascript::ast::ImportDecl {
        span: Default::default(),
        specifiers: imported_items
            .into_iter()
            .map(|item| swc_ecmascript::ast::ImportSpecifier::Named(swc_ecmascript::ast::ImportNamedSpecifier {
                span: Default::default(),
                local: as_ident(item),
                imported: None,
                is_type_only: false,
            }))
            .collect(),
        src: Box::new(swc_ecmascript::ast::Str {
            span: Default::default(),
            value: module_name.into(),
            raw: None,
        }),
        type_only,
        with: None,
        phase: swc_ecmascript::ast::ImportPhase::Evaluation,
    }))
}

pub fn get_helper_object_interface() -> TsInterfaceDecl {
    TsInterfaceDecl {
        span: Default::default(),
        id: as_ident("SailsJsHelper"),
        declare: true,
        type_params: Some(Box::new(TsTypeParamDecl {
            span: Default::default(),
            params: vec![
                TsTypeParam {
                    span: Default::default(),
                    name: as_ident("T"),
                    is_in: false,
                    is_out: false,
                    is_const: false,
                    constraint: None,
                    default: None,
                },
                TsTypeParam {
                    span: Default::default(),
                    name: as_ident("R"),
                    is_in: false,
                    is_out: false,
                    is_const: false,
                    constraint: None,
                    default: None,
                },
            ],
        })),
        extends: vec![],
        body: TsInterfaceBody {
            span: Default::default(),
            body: vec![TsTypeElement::TsMethodSignature(TsMethodSignature {
                span: Default::default(),
                key: Box::new(swc_ecmascript::ast::Expr::Ident(as_ident("with"))),
                computed: false,
                optional: false,
                params: vec![TsFnParam::Ident(BindingIdent {
                    id: as_ident("input"),
                    type_ann: Some(Box::new(TsTypeAnn {
                        span: Default::default(),
                        type_ann: Box::new(TsType::TsTypeRef(TsTypeRef {
                            span: Default::default(),
                            type_name: swc_ecmascript::ast::TsEntityName::Ident(as_ident("T")),
                            type_params: None,
                        })),
                    })),
                })],
                type_ann: Some(Box::new(TsTypeAnn {
                    span: Default::default(),
                    type_ann: Box::new(TsType::TsTypeRef(TsTypeRef {
                        span: Default::default(),
                        type_name: swc_ecmascript::ast::TsEntityName::Ident(as_ident("Promise")),
                        type_params: Some(Box::new(TsTypeParamInstantiation {
                            span: Default::default(),
                            params: vec![Box::new(TsType::TsTypeRef(TsTypeRef {
                                span: Default::default(),
                                type_name: swc_ecmascript::ast::TsEntityName::Ident(as_ident("R")),
                                type_params: None,
                            }))],
                        })),
                    })),
                })),
                type_params: None,
            })],
        },
    }
}

pub(crate) struct SailsModelInfo {
    pub name: String,
    pub type_name: String,
}

fn is_valid_ident(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    let valid_first = first == '_' || first == '$' || first.is_ascii_alphabetic();
    if !valid_first {
        return false;
    }
    chars.all(|c| c == '_' || c == '$' || c.is_ascii_alphanumeric())
}

// declare interface SailsObjectModels {
//   ModelName: ModelAccessor<ModelTypeName>;
// }
pub fn get_sails_object_models_interface(models: &[SailsModelInfo]) -> TsInterfaceDecl {
    TsInterfaceDecl {
        span: Default::default(),
        id: as_ident("SailsObjectModels"),
        declare: true,
        type_params: None,
        extends: vec![],
        body: TsInterfaceBody {
            span: Default::default(),
            body: models
                .iter()
                .map(|model| {
                    let (key, computed) = if is_valid_ident(&model.name) {
                        (
                            Box::new(swc_ecmascript::ast::Expr::Ident(as_ident(&model.name))),
                            false,
                        )
                    } else {
                        (
                            Box::new(swc_ecmascript::ast::Expr::Lit(swc_ecmascript::ast::Lit::Str(
                                swc_ecmascript::ast::Str {
                                    span: Default::default(),
                                    value: model.name.clone().into(),
                                    raw: None,
                                },
                            ))),
                            true,
                        )
                    };

                    TsTypeElement::TsPropertySignature(TsPropertySignature {
                        span: Default::default(),
                        readonly: false,
                        key,
                        computed,
                        optional: false,
                        type_ann: Some(Box::new(TsTypeAnn {
                            span: Default::default(),
                            type_ann: Box::new(TsType::TsTypeRef(TsTypeRef {
                                span: Default::default(),
                                type_name: swc_ecmascript::ast::TsEntityName::Ident(as_ident(
                                    "ModelAccessor",
                                )),
                                type_params: Some(Box::new(TsTypeParamInstantiation {
                                    span: Default::default(),
                                    params: vec![Box::new(TsType::TsTypeRef(TsTypeRef {
                                        span: Default::default(),
                                        type_name: swc_ecmascript::ast::TsEntityName::Ident(
                                            as_ident(&model.type_name),
                                        ),
                                        type_params: None,
                                    }))],
                                })),
                            })),
                        })),
                    })
                })
                .collect(),
        },
    }
}

// declare interface SailsObject {
//   helpers: HelpersObject;
//   models: SailsObjectModels;
// }
pub fn get_sails_object() -> ExportDecl {
    ExportDecl {
        span: Default::default(),
        decl: Decl::TsInterface(Box::new(TsInterfaceDecl {
            span: Default::default(),
            id: as_ident("SailsObject"),
            declare: true,
            type_params: None,
            extends: vec![],
            body: TsInterfaceBody {
                span: Default::default(),
                body: vec![
                    TsTypeElement::TsPropertySignature(TsPropertySignature {
                        span: Default::default(),
                        readonly: false,
                        key: Box::new(swc_ecmascript::ast::Expr::Ident(as_ident("helpers"))),
                        computed: false,
                        optional: false,
                        type_ann: Some(Box::new(TsTypeAnn {
                            span: Default::default(),
                            type_ann: Box::new(TsType::TsTypeRef(TsTypeRef {
                                span: Default::default(),
                                type_name: swc_ecmascript::ast::TsEntityName::Ident(as_ident(
                                    "HelpersObject",
                                )),
                                type_params: None,
                            })),
                        })),
                    }),
                    TsTypeElement::TsPropertySignature(TsPropertySignature {
                        span: Default::default(),
                        readonly: false,
                        key: Box::new(swc_ecmascript::ast::Expr::Ident(as_ident("models"))),
                        computed: false,
                        optional: false,
                        type_ann: Some(Box::new(TsTypeAnn {
                            span: Default::default(),
                            type_ann: Box::new(TsType::TsTypeRef(TsTypeRef {
                                span: Default::default(),
                                type_name: swc_ecmascript::ast::TsEntityName::Ident(as_ident(
                                    "SailsObjectModels",
                                )),
                                type_params: None,
                            })),
                        })),
                    }),
                ],
            },
        })),
    }
}

// declare global {
//   namespace NodeJS {
//     interface Global {
//       sails: SailsObject;
//     }
//   }
// }
pub fn get_global_namespace_declarations() -> TsModuleDecl {
    TsModuleDecl {
        span: Default::default(),
        declare: true,
        global: true,
        namespace: false,
        id: swc_ecmascript::ast::TsModuleName::Ident(as_ident("global")),
        body: Some(TsNamespaceBody::TsModuleBlock(TsModuleBlock {
            span: Default::default(),
            body: vec![TsModuleDecl {
                span: Default::default(),
                declare: false,
                global: false,
                namespace: true,
                id: swc_ecmascript::ast::TsModuleName::Ident(as_ident("NodeJS")),
                body: Some(TsNamespaceBody::TsModuleBlock(TsModuleBlock {
                    span: Default::default(),
                    body: vec![TsInterfaceDecl {
                        span: Default::default(),
                        id: as_ident("Global"),
                        declare: false,
                        type_params: None,
                        extends: vec![],
                        body: TsInterfaceBody {
                            span: Default::default(),
                            body: vec![TsTypeElement::TsPropertySignature(TsPropertySignature {
                                span: Default::default(),
                                readonly: false,
                                key: Box::new(swc_ecmascript::ast::Expr::Ident(as_ident("sails"))),
                                computed: false,
                                optional: false,
                                type_ann: Some(Box::new(TsTypeAnn {
                                    span: Default::default(),
                                    type_ann: Box::new(TsType::TsTypeRef(TsTypeRef {
                                        span: Default::default(),
                                        type_name: swc_ecmascript::ast::TsEntityName::Ident(as_ident(
                                            "SailsObject",
                                        )),
                                        type_params: None,
                                    })),
                                })),
                            })],
                        },
                    }
                    .into()],
                })),
            }
            .into()],
        })),
    }
}

// declare global {
//   export var sails: SailsObject;
// }
pub fn get_global_declarations() -> TsModuleDecl {
    TsModuleDecl {
        span: Default::default(),
        declare: true,
        global: true,
        namespace: false,
        id: swc_ecmascript::ast::TsModuleName::Ident(as_ident("global")),
        body: Some(TsNamespaceBody::TsModuleBlock(TsModuleBlock {
            span: Default::default(),
            body: vec![ModuleItem::ModuleDecl(
                swc_ecmascript::ast::ModuleDecl::ExportDecl(ExportDecl {
                    span: Default::default(),
                    decl: Decl::Var(Box::new(VarDecl {
                        span: Default::default(),
                        kind: VarDeclKind::Var,
                        declare: false,
                        decls: vec![VarDeclarator {
                            span: Default::default(),
                            name: Pat::Ident(BindingIdent {
                                id: as_ident("sails"),
                                type_ann: Some(Box::new(TsTypeAnn {
                                    span: Default::default(),
                                    type_ann: Box::new(TsType::TsTypeRef(TsTypeRef {
                                        span: Default::default(),
                                        type_name: swc_ecmascript::ast::TsEntityName::Ident(
                                            as_ident("SailsObject"),
                                        ),
                                        type_params: None,
                                    })),
                                })),
                            }),
                            init: None,
                            definite: false,
                        }],
                        ctxt: Default::default(),
                    })),
                }),
            )],
        })),
    }
}

// declare global {
//   var ModelName: ModelAccessor<ModelTypeName>;
// }
pub fn get_global_model_accessors(models: &[SailsModelInfo]) -> TsModuleDecl {
    let mut body: Vec<ModuleItem> = Vec::new();

    for model in models {
        if !is_valid_ident(&model.name) {
            continue;
        }

        body.push(ModuleItem::Stmt(swc_ecmascript::ast::Stmt::Decl(
            Decl::Var(Box::new(VarDecl {
                span: Default::default(),
                kind: VarDeclKind::Var,
                declare: false,
                decls: vec![VarDeclarator {
                    span: Default::default(),
                    name: Pat::Ident(BindingIdent {
                        id: as_ident(&model.name),
                        type_ann: Some(Box::new(TsTypeAnn {
                            span: Default::default(),
                            type_ann: Box::new(TsType::TsTypeRef(TsTypeRef {
                                span: Default::default(),
                                type_name: swc_ecmascript::ast::TsEntityName::Ident(as_ident(
                                    "ModelAccessor",
                                )),
                                type_params: Some(Box::new(TsTypeParamInstantiation {
                                    span: Default::default(),
                                    params: vec![Box::new(TsType::TsTypeRef(TsTypeRef {
                                        span: Default::default(),
                                        type_name: swc_ecmascript::ast::TsEntityName::Ident(
                                            as_ident(&model.type_name),
                                        ),
                                        type_params: None,
                                    }))],
                                })),
                            })),
                        })),
                    }),
                    init: None,
                    definite: false,
                }],
                ctxt: Default::default(),
            })),
        )));
    }

    TsModuleDecl {
        span: Default::default(),
        declare: true,
        global: true,
        namespace: false,
        id: swc_ecmascript::ast::TsModuleName::Ident(as_ident("global")),
        body: Some(TsNamespaceBody::TsModuleBlock(TsModuleBlock {
            span: Default::default(),
            body,
        })),
    }
}

// interface ModelAccessor<T> {
//   find(criteria: any): Promise<T[]>;
//   findOne(criteria: any): Promise<T | undefined>;
//   create(data: any): Promise<T>;
// }
pub fn get_model_accessor_interface() -> TsInterfaceDecl {
    TsInterfaceDecl {
        span: Default::default(),
        id: as_ident("ModelAccessor"),
        declare: true,
        type_params: Some(Box::new(TsTypeParamDecl {
            span: Default::default(),
            params: vec![TsTypeParam {
                span: Default::default(),
                name: as_ident("T"),
                is_in: false,
                is_out: false,
                is_const: false,
                constraint: None,
                default: None,
            }],
        })),
        extends: vec![],
        body: TsInterfaceBody {
            span: Default::default(),
            body: vec![
                TsTypeElement::TsMethodSignature(TsMethodSignature {
                    span: Default::default(),
                    key: Box::new(swc_ecmascript::ast::Expr::Ident(as_ident("find"))),
                    computed: false,
                    optional: false,
                    params: vec![TsFnParam::Ident(BindingIdent {
                        id: as_ident("criteria"),
                        type_ann: Some(Box::new(TsTypeAnn {
                            span: Default::default(),
                            type_ann: Box::new(TsType::TsKeywordType(TsKeywordType {
                                span: Default::default(),
                                kind: TsKeywordTypeKind::TsAnyKeyword,
                            })),
                        })),
                    })],
                    type_ann: Some(Box::new(TsTypeAnn {
                        span: Default::default(),
                        type_ann: Box::new(TsType::TsTypeRef(TsTypeRef {
                            span: Default::default(),
                            type_name: swc_ecmascript::ast::TsEntityName::Ident(as_ident(
                                "Promise",
                            )),
                            type_params: Some(Box::new(TsTypeParamInstantiation {
                                span: Default::default(),
                                params: vec![Box::new(TsType::TsArrayType(TsArrayType {
                                    span: Default::default(),
                                    elem_type: Box::new(TsType::TsTypeRef(TsTypeRef {
                                        span: Default::default(),
                                        type_name: swc_ecmascript::ast::TsEntityName::Ident(
                                            as_ident("T"),
                                        ),
                                        type_params: None,
                                    })),
                                }))],
                            })),
                        })),
                    })),
                    type_params: None,
                }),
                TsTypeElement::TsMethodSignature(TsMethodSignature {
                    span: Default::default(),
                    key: Box::new(swc_ecmascript::ast::Expr::Ident(as_ident("findOne"))),
                    computed: false,
                    optional: false,
                    params: vec![TsFnParam::Ident(BindingIdent {
                        id: as_ident("criteria"),
                        type_ann: Some(Box::new(TsTypeAnn {
                            span: Default::default(),
                            type_ann: Box::new(TsType::TsKeywordType(TsKeywordType {
                                span: Default::default(),
                                kind: TsKeywordTypeKind::TsAnyKeyword,
                            })),
                        })),
                    })],
                    type_ann: Some(Box::new(TsTypeAnn {
                        span: Default::default(),
                        type_ann: Box::new(TsType::TsTypeRef(TsTypeRef {
                            span: Default::default(),
                            type_name: swc_ecmascript::ast::TsEntityName::Ident(as_ident(
                                "Promise",
                            )),
                            type_params: Some(Box::new(TsTypeParamInstantiation {
                                span: Default::default(),
                                params: vec![Box::new(TsType::TsUnionOrIntersectionType(
                                    TsUnionOrIntersectionType::TsUnionType(TsUnionType {
                                        span: Default::default(),
                                        types: vec![
                                            Box::new(TsType::TsTypeRef(TsTypeRef {
                                                span: Default::default(),
                                                type_name: swc_ecmascript::ast::TsEntityName::Ident(
                                                    as_ident("T"),
                                                ),
                                                type_params: None,
                                            })),
                                            Box::new(TsType::TsKeywordType(TsKeywordType {
                                                span: Default::default(),
                                                kind: TsKeywordTypeKind::TsUndefinedKeyword,
                                            })),
                                        ],
                                    }),
                                ))],
                            })),
                        })),
                    })),
                    type_params: None,
                }),
                TsTypeElement::TsMethodSignature(TsMethodSignature {
                    span: Default::default(),
                    key: Box::new(swc_ecmascript::ast::Expr::Ident(as_ident("create"))),
                    computed: false,
                    optional: false,
                    params: vec![TsFnParam::Ident(BindingIdent {
                        id: as_ident("data"),
                        type_ann: Some(Box::new(TsTypeAnn {
                            span: Default::default(),
                            type_ann: Box::new(TsType::TsKeywordType(TsKeywordType {
                                span: Default::default(),
                                kind: TsKeywordTypeKind::TsAnyKeyword,
                            })),
                        })),
                    })],
                    type_ann: Some(Box::new(TsTypeAnn {
                        span: Default::default(),
                        type_ann: Box::new(TsType::TsTypeRef(TsTypeRef {
                            span: Default::default(),
                            type_name: swc_ecmascript::ast::TsEntityName::Ident(as_ident(
                                "Promise",
                            )),
                            type_params: Some(Box::new(TsTypeParamInstantiation {
                                span: Default::default(),
                                params: vec![Box::new(TsType::TsTypeRef(TsTypeRef {
                                    span: Default::default(),
                                    type_name: swc_ecmascript::ast::TsEntityName::Ident(as_ident(
                                        "T",
                                    )),
                                    type_params: None,
                                }))],
                            })),
                        })),
                    })),
                    type_params: None,
                }),
            ],
        },
    }
}
