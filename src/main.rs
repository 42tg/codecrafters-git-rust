use std::fs;

use clap::Args;
use clap::Subcommand;
use git_starter_rust::GitObject;

use clap::Parser;

#[derive(Debug, Parser)] 
#[command(name = "myGit")]
#[command(about = "A example Implementation of Git", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init,
    CatFile(CatFileArgs),
    HashObject(HashObjectArgs),
    LsTree(LsTreeArgs),
}

#[derive(Debug, Args)]
struct CatFileArgs {
    #[arg(short = 'p', required = true)]
    object: String,
}

#[derive(Debug, Args)]
struct HashObjectArgs {
    #[arg(short='w', required = true)]
    path: String,
}

#[derive(Debug, Args)]
struct LsTreeArgs {
    #[arg(long="name-only", required = true)]
    sha: String,
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();
            println!("Initialized git directory");
        }
        Commands::CatFile(args) => {
            let item = GitObject::from_hash(&args.object);
            match item.decode() {
                Ok(output) => output.print().unwrap(),
                Err(_) => println!("Failed to Decode File"),
            }
        }
        Commands::HashObject(args) => {
            let item = GitObject::from_file(&args.path);            
             match item.encode() {
                Ok(output) => print!("{}", output),
                Err(_) => println!("Failed to Encode File"),
             }
        }
        Commands::LsTree(args) => {
            let item = GitObject::from_hash(&args.sha);
            match item.decode() {
                Ok(output) => {
                    output.print().unwrap();
                },
                Err(e) => println!("Failed to Decode File\n{}", e),
            }
        }
    }
}
