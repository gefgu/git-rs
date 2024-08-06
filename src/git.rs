use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use hex::{self};
use sha1::{Digest, Sha1};
use std::fs;
use std::io::{prelude::*, Error};

#[derive(Debug)]
enum GitObjects {
    Tree,
    Blob,
}

#[derive(Debug)]
struct TreeEntry {
    mode: String,
    file_type: GitObjects,
    sha_hash: Vec<u8>,
    file_name: String,
}

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
            "hash-object" => Git::parse_hash_object_args(args),
            "ls-tree" => Git::parse_ls_tree_args(args),
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

    fn parse_hash_object_args(args: Vec<String>) {
        if args.len() < 2 {
            println!("No arguments given.");
            return;
        }

        match args[2].as_str() {
            "-w" => {
                if let Some(file_name) = args.get(3) {
                    Git::hash_object(file_name);
                } else {
                    println!("hash-object -w is lacking the 'file_name' argument.");
                }
            }
            _ => println!("unknown command: hash-object {}", args[2]),
        }
    }

    fn parse_ls_tree_args(args: Vec<String>) {
        if args.len() < 2 {
            println!("No arguments given.");
            return;
        }

        match args[2].as_str() {
            "--name-only" => {
                if let Some(tree_sha) = args.get(3) {
                    Git::read_tree_object(tree_sha, true);
                } else {
                    println!("ls-tree --name_only is lacking the 'tree_sha' argument.");
                }
            }
            _ => println!("unknown command: ls-tree {}", args[2]),
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
        let file_path = Git::get_object_path_from_hash(file_name);
        if let Ok(mut file_content) = Self::decode_object_file_to_string(&file_path) {
            if let Some(split_point) = file_content.find('\0') {
                let content = file_content.split_off(split_point + 1); // don't include \0
                print!("{content}");
            } else {
                println!("File {file_name} isn't a proper file");
            }
        } else {
            println!("Unable to read object file {file_name}");
        }
    }

    fn hash_object(file_name: &str) {
        if let Ok(file_content) = fs::read_to_string(file_name) {
            let byte_size = file_content.bytes().len();
            let content = format!("blob {byte_size}\0{file_content}");
            let hash = Git::get_hash(&content);

            let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
            let _ = e.write_all(content.as_bytes());
            let compressed = e.finish().unwrap_or_default();
            Git::write_blob_object(&hash, compressed);
        } else {
            println!("Unable to read file {file_name}");
        }
    }

    fn read_tree_object(tree_hash: &str, name_only: bool) {
        let file_path = Git::get_object_path_from_hash(tree_hash);
        let mut objects: Vec<TreeEntry> = Vec::new();
        if let Ok(file_content) = Self::decode_object_file_to_bytes(&file_path) {
            let mut starting_byte = 0;
            for (idx, bt) in file_content.iter().enumerate() {
                if *bt == b'\0' {
                    starting_byte = idx + 1;
                    break;
                }
            }

            while starting_byte < file_content.len() {
                let (_starting_byte, new_object) =
                    Git::parse_tree_object_row(starting_byte, &file_content).unwrap();
                starting_byte = _starting_byte;
                objects.push(new_object);
                if let Ok((_starting_byte, new_object)) =
                    Git::parse_tree_object_row(starting_byte, &file_content)
                {
                    objects.push(new_object);
                    starting_byte = _starting_byte;
                } else {
                    println!("{file_path} isn't a proper tree file");
                    break;
                }
                // break;
            }

            Git::print_tree_objects(objects, name_only);
        } else {
            println!("Unable to read object file {tree_hash}");
        }
    }

    // endregion

    // region: pretty priting function

    fn print_tree_objects(objs: Vec<TreeEntry>, name_only: bool) {
        if name_only {
            for obj in objs {
                println!("{}", obj.file_name);
            }
        } else {
            for obj in objs {
                let mut s = format!("{}", obj.mode);
                match obj.file_type {
                    GitObjects::Tree => {
                        s = format!("{s} tree");
                    }
                    GitObjects::Blob => {
                        s = format!("{s} blob");
                    }
                }
                s = format!("{s} {}", hex::encode(obj.sha_hash));
                s = format!("{s} {}", obj.file_name);
                println!("{s}");
            }
        }
    }

    // endregion

    // region: helper functions

    fn decode_object_file_to_bytes(file_path: &str) -> Result<Vec<u8>, Error> {
        let file_content = fs::read(file_path)?;
        let mut decoder = ZlibDecoder::new(&file_content[..]);
        let mut bytes_to_read = Vec::new();
        decoder.read_to_end(&mut bytes_to_read)?;
        return Ok(bytes_to_read);
    }

    fn decode_object_file_to_string(file_path: &str) -> Result<String, Error> {
        let file_content = fs::read(file_path)?;
        let mut decoder = ZlibDecoder::new(&file_content[..]);
        let mut string_to_read = String::new();
        decoder.read_to_string(&mut string_to_read)?;
        return Ok(string_to_read);
    }

    fn get_object_path_from_hash(hash: &str) -> String {
        let folder_name = Git::get_object_dir_name_from_hash(hash);
        let file_name = Git::get_object_file_name_from_hash(hash);
        let file_path = format!("{folder_name}/{file_name}");
        return file_path;
    }

    fn get_object_dir_name_from_hash(hash: &str) -> String {
        let header = hash.get(0..2).unwrap_or_default();
        format!(".git/objects/{header}")
    }

    fn get_object_file_name_from_hash(hash: &str) -> &str {
        hash.get(2..).unwrap_or_default()
    }

    fn get_hash(content: &str) -> String {
        let mut hasher = Sha1::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        let hex_string = hex::encode(result);
        return hex_string;
    }

    fn write_blob_object(hash: &str, content: Vec<u8>) {
        let object_path = Git::get_object_path_from_hash(&hash);
        let folder = Git::get_object_dir_name_from_hash(&hash);

        if let Err(_) = fs::read_dir(&folder) {
            fs::create_dir(&folder).unwrap();
        }

        if let Ok(_) = fs::write(object_path, content) {
            print!("{hash}");
        } else {
            println!("Error creating blob object");
        }
    }

    fn parse_tree_object_row(
        starting_byte: usize,
        bytes: &Vec<u8>,
    ) -> Result<(usize, TreeEntry), Error> {
        let mut end_mode_byte = starting_byte;
        for (idx, bt) in bytes[starting_byte..].iter().enumerate() {
            if *bt == b' ' {
                end_mode_byte = starting_byte + (idx - 1);
                break;
            }
        }
        let mut mode = String::new();
        (&bytes[(starting_byte)..=(end_mode_byte)]).read_to_string(&mut mode)?;

        let start_name_byte = end_mode_byte + 2;
        let mut end_name_byte = start_name_byte;

        for (idx, bt) in bytes[start_name_byte..].iter().enumerate() {
            if *bt == b'\0' {
                end_name_byte = start_name_byte + (idx - 1);
                break;
            }
        }

        let mut name = String::new();
        (&bytes[start_name_byte..=end_name_byte]).read_to_string(&mut name)?;

        let sha_hash = &bytes[(end_name_byte + 1)..=(end_name_byte + 20)];

        let file_type = match mode.as_str() {
            "40000" => GitObjects::Tree,
            _ => GitObjects::Blob,
        };

        return Ok((
            end_name_byte + 22,
            TreeEntry {
                mode,
                file_name: name,
                file_type,
                sha_hash: sha_hash.to_vec(),
            },
        ));
    }

    // endregions
}
