use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use hex;
use sha1::{Digest, Sha1};
use std::fs;
use std::io::prelude::*;

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
        if let Ok(file_content) = fs::read(file_path) {
            let mut decoder = ZlibDecoder::new(&file_content[..]);
            let mut s = String::new();
            decoder.read_to_string(&mut s).unwrap();
            if let Some(split_point) = s.find('\0') {
                let content = s.split_off(split_point + 1); // don't include \0
                print!("{content}");
            } else {
                println!("File {file_name} isn't a proper file");
            }
        } else {
            println!("Unable to read file {file_name}");
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

    // endregion

    // region: helper functions

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

        fs::write(object_path, content).unwrap();
        // if let Ok(_) = fs::write(object_path, content) {
        //     print!("{hash}");
        // } else {
        //     println!("Error creating blob object");
        // }
    }

    // endregions
}
