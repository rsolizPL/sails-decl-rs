use std::collections::HashMap;
use std::path::PathBuf;
use swc_common::Spanned;
use swc_common::source_map::DefaultSourceMapGenConfig;
use swc_common::sync::Lrc;

use swc_common::{
    FileName, SourceMap,
    errors::{ColorConfig, Handler},
};
use swc_ecma_codegen::text_writer::JsWriter;
use swc_ecma_codegen::{Config, Emitter};
use swc_ecma_parser::{Lexer, Parser, StringInput, Syntax};
use swc_ecmascript::ast::{
    Expr, Ident, Lit, Module, Str, TsEntityName, TsKeywordType, TsPropertySignature, TsType, TsTypeAliasDecl, TsTypeAnn, TsTypeElement, TsTypeLit, TsTypeParamInstantiation, TsTypeRef
};

use crate::literal_declarations::{get_global_declarations, get_global_namespace_declarations, get_helper_object_interface, get_sails_object};
use crate::util::{EmittedCode, find_module_exports, get_prop_as_str, ts_type_from_attribute};

pub fn build_tree(
    helpers: &[PathBuf],
    helpers_folder: &PathBuf,
    cm: Lrc<SourceMap>,
) -> Vec<SailsDeclHelperTreeNode> {
    // Initial pass: make sure we are only looking at paths relative to the root folder
    let relative_paths: Vec<PathBuf> = helpers
        .iter()
        .filter_map(|p| p.strip_prefix(&helpers_folder).ok().map(|s| s.to_path_buf()))
        .collect();

    build_tree_recursive(&relative_paths, &helpers_folder, cm)
}

fn build_tree_recursive(
    paths: &[PathBuf],
    current_base: &PathBuf,
    cm: Lrc<SourceMap>,
) -> Vec<SailsDeclHelperTreeNode> {
    let mut nodes = Vec::new();
    let mut groups: HashMap<String, Vec<PathBuf>> = HashMap::new();

    // Group paths by their next immediate component
    for path in paths {
        if let Some(first) = path.components().next() {
            let name = first.as_os_str().to_string_lossy().into_owned();
            groups.entry(name).or_default().push(path.clone());
        }
    }

    for (name, paths_in_group) in groups {
        let full_path = current_base.join(&name);

        // Check if this group represents a single leaf node (the file itself)
        // This happens when one of the paths in the group is exactly the 'name'
        if paths_in_group.iter().any(|p| p.components().count() == 1) {
            if full_path.extension().and_then(|s| s.to_str()) == Some("js") {
                match get_helper_info(full_path, cm.clone()) {
                    Ok(helper_info) => nodes.push(SailsDeclHelperTreeNode::Helper(helper_info)),
                    Err(e) => eprintln!("Failed to parse helper: {:?}", e),
                }
            }
        } else {
            // Otherwise, it's a directory. Strip the prefix and recurse.
            let sub_paths: Vec<PathBuf> = paths_in_group
                .into_iter()
                .filter_map(|p| p.strip_prefix(&name).ok().map(|s| s.to_path_buf()))
                .collect();

            if !sub_paths.is_empty() {
                let children = build_tree_recursive(&sub_paths, &full_path, cm.clone());
                nodes.push(SailsDeclHelperTreeNode::Directory(SailsDeclHelperDirectory {
                    name: normalize_name(&name),
                    children,
                }));
            }
        }
    }

    nodes
}

fn normalize_name(name: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for c in name.chars() {
        if c == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    // if result ends in .js, remove that
    if result.ends_with(".js") {
        result.truncate(result.len() - 3);
    }

    result
}

#[derive(Clone)]
pub struct SailsHelperInfo {
    pub path: PathBuf,
    pub name: String,
    pub return_type: Option<TsType>,
    pub input_type: Option<TsType>,
}

#[derive(Debug)]
pub enum GenHelperDeclError {
    ParseError,
    IsNotCommonJsModule,
    IsNotHelper,
    CommonJsModuleDoesNotExportObject,
}

pub fn get_helper_info(helper: PathBuf, cm: Lrc<SourceMap>) -> Result<SailsHelperInfo, GenHelperDeclError> {
    let code = std::fs::read_to_string(&helper).map_err(|_| GenHelperDeclError::ParseError)?;

    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

    let file = cm.new_source_file(FileName::Real(helper.clone()).into(), code);

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
        return Err(GenHelperDeclError::ParseError);
    }

    let module = module_result.unwrap();

    let module_exports_obj =
        find_module_exports(module).ok_or(GenHelperDeclError::CommonJsModuleDoesNotExportObject)?;

    let _fn_obj = module_exports_obj
        .props
        .iter()
        .find_map(|prop| {
            let _key_value_prop = prop.as_prop().and_then(|p| p.as_key_value())?;
            let key_name = get_prop_as_str(&_key_value_prop.key)?;

            if key_name != "fn" {
                return None;
            }

            _key_value_prop.value.as_fn_expr().cloned()
        })
        .ok_or(GenHelperDeclError::IsNotHelper)?;

    let inputs_obj = module_exports_obj.props.iter().find_map(|prop| {
        let _key_value_prop = prop.as_prop().and_then(|p| p.as_key_value())?;
        let key_name = get_prop_as_str(&_key_value_prop.key)?;

        if key_name != "inputs" {
            return None;
        }

        _key_value_prop.value.as_object().cloned()
    });

    if inputs_obj.is_none() {
        // I think sails still technically allows helpers without inputs, so
        // we'll just return a default helper declaration with no input type instead of erroring out
        return Ok(SailsHelperInfo {
            path: helper.clone(),
            name: normalize_name(helper.file_name().unwrap().to_str().unwrap()),
            return_type: None,
            input_type: None,
        });
    }

    let inputs_obj = inputs_obj.unwrap();

    let mut inputs: Vec<TsTypeElement> = vec![];

    for input in &inputs_obj.props {
        let input_pair = match input.as_prop().and_then(|p| p.as_key_value()) {
            Some(prop) => prop,
            None => continue,
        };

        let _input_key_ident = match input_pair.key.as_ident() {
            Some(ident) => ident,
            None => continue,
        };

        let _input_value_obj = match input_pair.value.as_object() {
            Some(obj) => obj,
            None => continue,
        };

        let input_type_info = match ts_type_from_attribute(_input_value_obj) {
            Some(info) => info,
            None => continue,
        };

        inputs.push(TsTypeElement::TsPropertySignature(TsPropertySignature {
            span: input_pair.key.span(),
            readonly: false,
            key: Box::new(Expr::Lit(Lit::Str(Str {
                span: input_pair.key.span(),
                value: _input_key_ident.sym.as_str().into(),
                raw: None,
            }))),
            computed: true,
            optional: input_type_info.required,
            type_ann: Some(Box::new(TsTypeAnn {
                span: Default::default(),
                type_ann: Box::new(input_type_info.ts_type),
            })),
        }));
    }

    return Ok(SailsHelperInfo {
        path: helper.clone(),
        name: normalize_name(helper.file_name().unwrap().to_str().unwrap()),
        return_type: None,
        input_type: Some(TsType::TsTypeLit(TsTypeLit {
            span: Default::default(),
            members: inputs,
        })),
    });
}

pub struct SailsDeclHelperDirectory {
    name: String,
    children: Vec<SailsDeclHelperTreeNode>,
}

pub enum SailsDeclHelperTreeNode {
    Helper(SailsHelperInfo),
    Directory(SailsDeclHelperDirectory),
}

pub struct SailsDeclHelperTree {
    root: Vec<SailsDeclHelperTreeNode>,
}

impl SailsDeclHelperTree {
    pub fn new(helpers: &Vec<PathBuf>, helpers_folder: &PathBuf, cm: Lrc<SourceMap>) -> Self {
        SailsDeclHelperTree {
            root: build_tree(helpers, helpers_folder, cm),
        }
    }

    pub fn get_root(&self) -> &Vec<SailsDeclHelperTreeNode> {
        &self.root
    }

    pub fn get_all_helpers(&self) -> Vec<&SailsHelperInfo> {
        let mut helpers = Vec::new();
        self.collect_helpers(&self.root, &mut helpers);
        helpers
    }

    fn collect_helpers<'a>(
        &self,
        nodes: &'a Vec<SailsDeclHelperTreeNode>,
        helpers: &mut Vec<&'a SailsHelperInfo>,
    ) {
        for node in nodes {
            match node {
                SailsDeclHelperTreeNode::Helper(helper_info) => helpers.push(helper_info),
                SailsDeclHelperTreeNode::Directory(dir) => self.collect_helpers(&dir.children, helpers),
            }
        }
    }
}

fn gen_decl_from_node(node: &SailsDeclHelperTreeNode) -> TsTypeElement {
    match node {
        SailsDeclHelperTreeNode::Helper(helper_info) => {
            TsTypeElement::TsPropertySignature(TsPropertySignature {
                span: Default::default(),
                readonly: true,
                key: Box::new(Expr::Lit(Lit::Str(Str {
                    span: Default::default(),
                    value: helper_info.name.clone().into(),
                    raw: None,
                }))),
                computed: true,
                optional: false,
                type_ann: Some(Box::new(TsTypeAnn {
                    span: Default::default(),
                    type_ann: Box::new(TsType::TsTypeRef(TsTypeRef {
                        span: Default::default(),
                        type_name: TsEntityName::Ident(Ident {
                            span: Default::default(),
                            ctxt: Default::default(),
                            sym: "SailsJsHelper".into(),
                            optional: false,
                        }),
                        type_params: Some(Box::new(TsTypeParamInstantiation {
                            span: Default::default(),
                            params: vec![
                                Box::new(helper_info.input_type.clone().unwrap_or(TsType::TsKeywordType(
                                    TsKeywordType {
                                        span: Default::default(),
                                        kind: swc_ecmascript::ast::TsKeywordTypeKind::TsAnyKeyword,
                                    },
                                ))),
                                Box::new(helper_info.return_type.clone().unwrap_or(TsType::TsKeywordType(
                                    TsKeywordType {
                                        span: Default::default(),
                                        kind: swc_ecmascript::ast::TsKeywordTypeKind::TsAnyKeyword,
                                    },
                                ))),
                            ],
                        })),
                    })),
                })),
            })
        }
        SailsDeclHelperTreeNode::Directory(SailsDeclHelperDirectory { name, children }) => {
            TsTypeElement::TsPropertySignature(TsPropertySignature {
                span: Default::default(),
                readonly: true,
                key: Box::new(Expr::Lit(Lit::Str(Str {
                    span: Default::default(),
                    value: name.clone().into(),
                    raw: None,
                }))),
                computed: true,
                optional: false,
                type_ann: Some(Box::new(TsTypeAnn {
                    span: Default::default(),
                    type_ann: Box::new(TsType::TsTypeLit(TsTypeLit {
                        span: Default::default(),
                        members: children.iter().map(gen_decl_from_node).collect(),
                    })),
                })),
            })
        }
    }
}

fn gen_decl_from_tree(tree: SailsDeclHelperTree) -> TsType {
    TsType::TsTypeLit(TsTypeLit {
        span: Default::default(),
        members: tree.get_root().iter().map(gen_decl_from_node).collect(),
    })
}

pub fn gen_helpers_object_decl(tree: SailsDeclHelperTree) -> TsTypeAliasDecl {
    let decl = gen_decl_from_tree(tree);

    TsTypeAliasDecl {
        span: Default::default(),
        declare: true,
        id: Ident {
            span: Default::default(),
            ctxt: Default::default(),
            sym: "HelpersObject".into(),
            optional: false,
        },
        type_params: None,
        type_ann: Box::new(decl),
    }
}

pub fn emit_helpers_with_sourcemap(helpers: &Vec<PathBuf>, helpers_folder: &PathBuf, output_dts_path: &PathBuf) -> EmittedCode {
    // 1. Create the master SourceMap that will hold ALL files
    let cm: Lrc<SourceMap> = Default::default();

    // 2. Build the tree, passing the shared 'cm'
    // You'll need to update build_tree and get_helper_info to accept &cm
    let tree = SailsDeclHelperTree::new(helpers, helpers_folder, cm.clone());
    let decl = gen_helpers_object_decl(tree);

    let module = Module {
        span: Default::default(),
        body: vec![
            get_helper_object_interface().into(),
            decl.into(),
            get_sails_object().into(),
            get_global_namespace_declarations().into(),
            get_global_declarations().into(),
        ],
        shebang: None,
    };

    let mut buf = Vec::new();
    let mut src_map_buf = Vec::new();

    // 3. Emit the aggregated AST
    {
        let writer = JsWriter::new(cm.clone(), "\n", &mut buf, Some(&mut src_map_buf));
        let mut emitter = Emitter {
            cfg: Config::default().with_minify(false),
            cm: cm.clone(),
            comments: None,
            wr: writer,
        };

        emitter.emit_module(&module).unwrap();
    }

    let mut code = String::from_utf8(buf).expect("utf8");

    // 4. Generate the multi-source map
    let mut sourcemap = cm.build_source_map(&src_map_buf, None, DefaultSourceMapGenConfig {});
    
    // Set the output filename so the LSP knows which file this map belongs to
    let dts_name = output_dts_path.file_name().map(|n| n.to_string_lossy().into_owned());
    sourcemap.set_file(dts_name);

    let mut map_buf = Vec::new();
    sourcemap.to_writer(&mut map_buf).unwrap();
    let source_map_json = String::from_utf8(map_buf).unwrap();

    // 5. Append the mapping URL to the bottom of the .d.ts
    let map_file_name = format!("{}.map", output_dts_path.file_name().unwrap().to_str().unwrap());
    code.push_str(&format!("\n//# sourceMappingURL={}", map_file_name));

    EmittedCode { 
        code, 
        source_map: source_map_json 
    }
}