use std::path::PathBuf;

use clap::{Parser, Subcommand};
use std::time::{Instant, Duration};

#[derive(Parser)]
#[command(
    version = "0.1.0",
    author = "Rubyboat <me@rubyboat.net>",
    about = "A CLI tool to add type declarations to Sails.js projects"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(name = "run")]
    Run {
        #[arg(value_parser)]
        project_root: Option<PathBuf>,
        #[arg(short = 'i', long = "ignored-files", value_parser)]
        ignored_files: Vec<PathBuf>,
        #[arg(short = 'm', long = "model-dir", value_parser)]
        model_dir: Option<PathBuf>,
        #[arg(short = 'e', long = "helpers-dir", value_parser)]
        helpers_dir: Option<PathBuf>,
        #[arg(short = 't', long = "types-dir", value_parser)]
        types_dir: Option<PathBuf>
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Run {
            project_root,
            ignored_files,
            model_dir,
            helpers_dir,
            types_dir,
        }) => run(
            project_root,
            ignored_files,
            model_dir,
            helpers_dir,
            types_dir,
        ),
        None => run(&None, &vec![], &None, &None, &None),
    }
}

fn run(
    project_root: &Option<PathBuf>,
    ignored_files: &Vec<PathBuf>,
    model_dir: &Option<PathBuf>,
    helpers_dir: &Option<PathBuf>,
    types_dir: &Option<PathBuf>,
) {
    let overall_start = Instant::now();
    let cwd = std::env::current_dir().expect("Failed to get current directory");
    let project_root = project_root.as_ref().unwrap_or(&cwd);

    // Check for .sailsrc file in project_root
    let sailsrc_path = project_root.join(".sailsrc");
    if !sailsrc_path.exists() {
        eprintln!("Error: .sailsrc file not found in project root: {}", project_root.display());
        std::process::exit(1);
    }

    let model_dir = model_dir
        .as_ref()
        .unwrap_or(&project_root.join("api/models"))
        .clone();
    let _helpers_dir = helpers_dir
        .as_ref()
        .unwrap_or(&project_root.join("api/helpers"))
        .clone();
    let types_dir = types_dir
        .as_ref()
        .unwrap_or(&project_root.join("typings"))
        .clone();
    let models_types_dir = types_dir.join("models");
    
    // recursively find all .js files in the model_dir, excluding ignored_files
    let js_files = glob::glob(&format!("{}/**/*.js", model_dir.display()))
        .expect("Failed to read glob pattern")
        .filter_map(Result::ok)
        .filter(|path| {
            !ignored_files
                .iter()
                .any(|ignored| path.starts_with(ignored))
        })
        .collect::<Vec<_>>();
    let js_files_count = js_files.len();
    println!(
        "Found {} Models in {}",
        js_files_count,
        model_dir.display()
    );

    for js_file in js_files {
        let code = std::fs::read_to_string(&js_file).expect("Failed to read model file");
        let name = js_file.file_stem().unwrap().to_string_lossy().to_string();
        match sails_decl_core::model::gen_decl(code, name, Some(js_file.clone())) {
            Ok(decl) => {
                let new_path = models_types_dir.join(js_file.strip_prefix(&model_dir).unwrap());
                let declaration_path = new_path.with_extension("d.ts");
                let decl_code = sails_decl_core::model::emit_with_source_map(decl, &declaration_path);
                // same relative path as the js file, but with .d.ts extension and in the types_dir
                std::fs::create_dir_all(new_path.parent().unwrap()).expect("Failed to create directories for output file");
                std::fs::write(&declaration_path, decl_code.code).expect("Failed to write declaration file");
                std::fs::write(new_path.with_extension("d.ts.map"), decl_code.source_map).expect("Failed to write source map file");
                println!("Generated declaration for {} at {}", js_file.strip_prefix(&model_dir).unwrap().display(), new_path.strip_prefix(&types_dir).unwrap().display());
            }
            Err(e) => eprintln!("Error processing {}: {:?}", js_file.display(), e),
        }
    }

    let total_duration = overall_start.elapsed();

    println!(
        "Processed {} models in {} ms",
        js_files_count,
        total_duration.as_millis()
    );
}
