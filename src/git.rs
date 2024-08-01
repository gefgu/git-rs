use std::fs;

pub struct Git {}

impl Git {
    pub fn parse_args(args: Vec<String>) {
        if args.len() < 1 {
            println!("No arguments given.");
            return;
        }

        match args[1].as_str() {
            "init" => Git::init_git_directory(),
            _ => println!("unknown command: {}", args[1]),
        }
    }

    fn init_git_directory() {
        fs::create_dir(".git").unwrap();
        fs::create_dir(".git/objects").unwrap();
        fs::create_dir(".git/refs").unwrap();
        fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
        println!("Initialized git directory")
    }
}
