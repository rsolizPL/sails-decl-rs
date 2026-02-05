use std::path::PathBuf;

use clap::{Parser, Subcommand};
use swc_ecma_codegen::{to_code};

#[derive(Parser)]
#[command(
    version = "0.1.0",
    author = "Rubyboat <me@rubyboat.net>",
    about = "A CLI tool to add type declarations to Sails.js projects",
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(name = "run")]
    Run{
        #[arg(short = 'p', long = "project-root", value_parser)]
        project_root: Option<PathBuf>,
        #[arg(short = 'i', long = "ignored-files", value_parser)]
        ignored_files: Vec<PathBuf>,
    },
}

fn main() {
    let model_file = r#"
    module.exports = {
        attributes: {
            name: { type: 'string', required: true },
            age: { type: 'number' },
            isActive: { type: 'boolean', defaultsTo: true },
            tags: {
                type: 'json',
                "$SD-type-hint": "{ color: string, label: string }[]",
            },
            profile: { model: 'profile' },
        }
    };"#.to_string();

    let model_name = "User".to_string();

    let decl = sails_decl_core::gen_decl_for_model(model_file, model_name).unwrap();

    println!("Generated Declaration: {}", to_code(&decl));
}
