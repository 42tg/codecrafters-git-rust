#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;

use std::io::prelude::*;
use flate2::read::ZlibDecoder;

fn decode_reader(bytes: Vec<u8>) -> std::io::Result<String> {
    let mut z = ZlibDecoder::new(&bytes[..]);
    let mut s = String::new();
    z.read_to_string(&mut s)?;
    Ok(s.find("\x00").and_then(|i| Some(&s[i+1..] as &str)).unwrap_or(&s[..]).to_string())
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    let args: Vec<String> = env::args().collect();
    if args[1] == "init" {
        fs::create_dir(".git").unwrap();
        fs::create_dir(".git/objects").unwrap();
        fs::create_dir(".git/refs").unwrap();
        fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();
        println!("Initialized git directory")
    } if args[1] == "cat-file" {
        let hash = &args[3];
        let (directory, file) = hash.split_at(2);
        let path = format!(".git/objects/{}/{}",directory, file);
        let contents = fs::read(path).unwrap();
        
        let output = decode_reader(contents).unwrap();
        
        
        print!("{}",output );

    } else {
        println!("unknown command: {}", args[1])
    }
}
