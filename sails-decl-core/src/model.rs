extern crate swc_common;
extern crate swc_ecma_parser;

use std::path::PathBuf;

use swc_common::Spanned;
use swc_common::source_map::{DefaultSourceMapGenConfig};
use swc_common::sync::Lrc;
use swc_common::{
    FileName, SourceMap,
    errors::{ColorConfig, Handler},
};
use swc_ecma_codegen::{Config, Emitter};
use swc_ecma_codegen::text_writer::JsWriter;
use swc_ecma_parser::{Parser, StringInput, Syntax, lexer::Lexer};
use swc_ecmascript::ast::{Script, TsInterfaceDecl};
use swc_ecmascript::ast::{
    Expr, Ident, Lit, Str, TsInterfaceBody, TsPropertySignature, TsTypeAnn, TsTypeElement,
};

use crate::util::{find_module_exports, get_prop_as_str, ts_type_from_attribute};
#[derive(Debug)]
pub enum GenDeclarationsError {
    ParseError,
    InvalidModel,
    IsNotCommonJsModule,
    CommonJsModuleDoesNotExportObject,
    SDTypeHintParseError,
}

pub struct ModelDecl {
    interface: Script,
    source_map: Lrc<SourceMap>,
}

pub fn gen_decl(code: String, model_name: String, file_path: Option<PathBuf>) -> Result<ModelDecl, GenDeclarationsError> {
    let cm: Lrc<SourceMap> = Default::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

    let file = cm.new_source_file(match file_path {
        Some(path) => FileName::Real(path),
        None => FileName::Anon,
     }.into(), code);

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

    let module_exports_obj =
        find_module_exports(module).ok_or(GenDeclarationsError::IsNotCommonJsModule)?;

    let mut elements: Vec<TsTypeElement> = vec![];

    let attributes_obj = module_exports_obj
        .props
        .iter()
        .find_map(|prop| {
            let _key_value_prop = prop.as_prop().and_then(|p| p.as_key_value())?;
            let key_name = get_prop_as_str(&_key_value_prop.key)?;

            if key_name != "attributes" {
                return None;
            }

            _key_value_prop.value.as_object().cloned()
        })
        .ok_or(GenDeclarationsError::InvalidModel)?;

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

        let attribute_type_info = match ts_type_from_attribute(_attr_value_obj) {
            Some(info) => info,
            None => continue,
        };

        elements.push(TsTypeElement::TsPropertySignature(TsPropertySignature {
            span: attribute_pair.key.span(),
            readonly: false,
            key: Box::new(Expr::Lit(Lit::Str(Str {
                span: attribute_pair.key.span(),
                value: _attr_key_ident.sym.as_str().into(),
                raw: None,
            }))),
            computed: true,
            optional: !attribute_type_info.required,
            type_ann: Some(Box::new(TsTypeAnn {
                span: Default::default(),
                type_ann: Box::new(attribute_type_info.ts_type),
            })),
        }));
    }

    Ok(ModelDecl {
        interface: Script { span: Default::default(), body: vec![
            TsInterfaceDecl {
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
            }.into()
        ], shebang: None },
        source_map: cm
    })
}

pub struct ModelCode {
    pub code: String,
    pub source_map: String,
}

pub fn emit_with_source_map(decl: ModelDecl, output_dts_path: &PathBuf) -> ModelCode {
    let mut buf = Vec::new();
    let mut src_map_buf = Vec::new();

    {
        let writer = JsWriter::new(decl.source_map.clone(), "\n", &mut buf, Some(&mut src_map_buf));
        let mut emitter = Emitter {
            cfg: Config::default().with_minify(false),
            cm: decl.source_map.clone(),
            comments: None,
            wr: writer,
        };

        emitter.emit_script(&decl.interface).unwrap();
    }

    let code = String::from_utf8(buf).expect("utf8");
    
    // SWC builds the sourcemap internally based on the spans you provided
    let mut sourcemap = decl.source_map.build_source_map(&src_map_buf, None, DefaultSourceMapGenConfig {});
    let mut map_buf = Vec::new();
    let dts_filename = output_dts_path
        .file_name()
        .map(|f| f.to_string_lossy().into_owned());
    sourcemap.set_file(dts_filename);
    sourcemap.to_writer(&mut map_buf).unwrap();
    let map_json = String::from_utf8(map_buf).unwrap();

    ModelCode {
        code,
        source_map: map_json,
    }
}
