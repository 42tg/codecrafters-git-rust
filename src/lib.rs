use std::fs::{self, File};
use std::marker::PhantomData;
use std::io::{prelude::*, BufReader, BufWriter};
use anyhow::Ok;
use flate2::read::{ZlibDecoder, ZlibEncoder};
use flate2::Compression;
use sha1::Digest;


pub struct ReadMode;
pub struct WriteMode;
 
pub struct GitObject<T> {
    phantom: PhantomData<T>,

    file_path: String,
    hash: String,
    file_content: Option<Vec<u8>>
}

impl GitObject<ReadMode> {
    pub fn from_hash(hash: &str) -> Self {
        let (directory, file) = hash.split_at(2);
        let path = format!(".git/objects/{}/{}",directory, file);

        Self {
            hash: hash.to_string(),
            file_path: path,
            phantom: PhantomData,
            file_content: None
        }
    }

    pub fn decode(&self) -> Result<String, anyhow::Error> {
        let contents = fs::read(&self.file_path)?;

        let mut z = ZlibDecoder::new(&contents[..]);
        let mut s = String::new();
        z.read_to_string(&mut s)?;
        let (_, tail) = s.split_once("\0").unwrap();

        Ok(tail.to_string())
    }
}

impl GitObject<WriteMode> {
    pub fn from_file(file_path: &str) -> Self {
        // Read the file
        let source_file = File::open(file_path).expect("File not found");
        let size = source_file.metadata().expect("File metadata not accessable").len();

        // Read the file into a buffer
        let mut reader = BufReader::new(source_file);
        let mut buffer = Vec::new();

        // Write the header
        buffer.extend("blob ".as_bytes());
        buffer.extend(size.to_string().as_bytes());
        buffer.push(0);

        // Write the content
        reader.read_to_end(&mut buffer).expect("Cannot write to buffer");

        // create hash
        let mut hasher = sha1::Sha1::default();
        hasher.update(&buffer);
        let hash = hasher.finalize();
        let hash_string = hex::encode(hash);


        let (directory, file) = hash_string.split_at(2);
        let path = format!(".git/objects/{}/{}", directory, file);
        
        Self {
            hash: hash_string,
            file_path: path,
            phantom: PhantomData,
            file_content: Some(buffer)
        }
    }

    pub fn encode(&self) -> Result<String, anyhow::Error> {
        let file_content = self.file_content.as_ref().unwrap();

        let mut zlib_reader = ZlibEncoder::new(BufReader::new(&file_content[..]), Compression::fast());

        let (directory, _) = self.hash.split_at(2);
        fs::create_dir_all(format!(".git/objects/{}", directory))?;

        let output_file = File::create(&self.file_path)?;
        std::io::copy(&mut zlib_reader, &mut BufWriter::new(output_file))?;

        Ok(self.hash.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        todo!("Do this");
    }
}
