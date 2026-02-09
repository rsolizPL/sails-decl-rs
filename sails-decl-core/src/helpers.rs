use std::path::PathBuf;

use swc_ecmascript::ast::TsType;

struct _SailsHelper {
  path: PathBuf,
  location: Vec<String>,
  return_type: Option<String>,
  input_type: Option<TsType>,
}

