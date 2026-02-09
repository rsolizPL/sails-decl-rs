use swc_ecmascript::ast::{
    BindingIdent, Decl, ExportDecl, Ident, ModuleItem, Pat, TsFnParam, TsInterfaceBody,
    TsInterfaceDecl, TsMethodSignature, TsModuleBlock, TsModuleDecl, TsNamespaceBody,
    TsPropertySignature, TsType, TsTypeAnn, TsTypeElement, TsTypeParam,
    TsTypeParamDecl, TsTypeParamInstantiation, TsTypeRef, VarDecl, VarDeclKind, VarDeclarator,
};

fn as_ident(sym: &str) -> Ident {
    Ident {
        span: Default::default(),
        ctxt: Default::default(),
        sym: sym.into(),
        optional: false,
    }
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

// declare interface SailsObject {
//     helpers: HelpersObject;
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
                body: vec![TsTypeElement::TsPropertySignature(TsPropertySignature {
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
                })],
            },
        })),
    }
}

// declare namespace NodeJS {
//     interface Global {
//         sails: SailsObject;
//     }
// }
pub fn get_global_namespace_declarations() -> TsModuleDecl {
    TsModuleDecl {
        span: Default::default(),
        declare: true,
        global: false,
        namespace: true,
        id: swc_ecmascript::ast::TsModuleName::Ident(as_ident("NodeJS")),
        body: Some(TsNamespaceBody::TsModuleBlock(TsModuleBlock {
            span: Default::default(),
            body: vec![
                TsInterfaceDecl {
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
                .into(),
            ],
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
