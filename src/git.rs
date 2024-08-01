use std::fs;

pub struct Git {}

impl Git {
    // region: parsing

    pub fn parse_args(args: Vec<String>) {
        if args.len() < 1 {
            println!("No arguments given.");
            return;
        }

        match args[1].as_str() {
            "init" => Git::init_git_directory(),
            "cat-file" => Git::parse_cat_file_args(args),
            _ => println!("unknown command: {}", args[1]),
        }
    }

    fn parse_cat_file_args(args: Vec<String>) {
        if args.len() < 2 {
            println!("No arguments given.");
            return;
        }

        match args[2].as_str() {
            "-p" => {
                if let Some(file_name) = args.get(3) {
                    Git::read_blob_object(file_name);
                } else {
                    println!("cat-file -p is lacking the 'file_name' argument.");
                }
            }
            _ => println!("unknown command: cat-file {}", args[2]),
        }
    }

    // endregion

    // region: git actions

    fn init_git_directory() {
        fs::create_dir(".git").unwrap();
        fs::create_dir(".git/objects").unwrap();
        fs::create_dir(".git/refs").unwrap();
        fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
        println!("Initialized git directory")
    }

    fn read_blob_object(file_name: &str) {
        if let Ok(file_content) = fs::read(file_name) {
            println!("{:?}", file_content);
        } else {
            println!("Unable to read file {file_name}");
        }
    }

    // endregion
}
