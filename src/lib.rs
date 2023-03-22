use std::fs::{self, File};
use std::marker::PhantomData;
use std::io::{prelude::*, BufReader, BufWriter, stdout, Cursor};
use anyhow::{Ok, anyhow};
use flate2::read::{ZlibDecoder, ZlibEncoder};
use flate2::Compression;
use sha1::Digest;


pub struct ReadMode;
pub struct WriteMode;
 
pub struct GitType {
    object_type: String,
    content: Vec<u8>
}

impl GitType {
    pub fn print(&self) -> Result<(), anyhow::Error>{
        match self.object_type.as_str() {
            "commit" => {
                let mut stdout = stdout();
                stdout.write_all(&self.content).unwrap();
                Ok(())
            },
            "blob" => {
                let mut stdout = stdout();
                stdout.write_all(&self.content).unwrap();
                Ok(())
            },
            "tree" => {
                let content_len = self.content.len();
                let mut reader = BufReader::new(Cursor::new(&self.content));
                while reader.stream_position()? < content_len as u64 {
                    //line looks like 100644<space>file_name.txt<null_byte><20_byte_sha1>
                    let mut buffer = Vec::new();
                    // read to 100644<space>
                    reader.read_until(' ' as u8, &mut buffer)?;
                    buffer.pop();
                    buffer.clear();
                    
                    // read to file_name.txt<null_byte>
                    reader.read_until(0, &mut buffer)?;
                    buffer.pop();
                    let file_name = String::from_utf8(buffer)?;

                    // Size of the sha1 hash
                    reader.seek_relative(20)?;

                    println!("{}", file_name);
                }
                Ok(())
            },
            _ => {
                Err(anyhow!(
                    "Unsupported Object Type: {}", self.object_type
                ))
            }
        }
    }
} 

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

    pub fn decode(&self) -> Result<GitType, anyhow::Error> {        
        let file = BufReader::new(File::open(&self.file_path)?);
        let decoder = ZlibDecoder::new(file);

        let mut reader = BufReader::new(decoder);
        let mut buffer = Vec::new();
        
        reader.read_until(' ' as u8, &mut buffer)?;
        
        // Remove Empty Item
        buffer.pop();
        let object_type = String::from_utf8(buffer.clone())?;

        //Reset buffer
        buffer.clear();
        reader.read_until(0, &mut buffer)?;
        buffer.pop();

        let size = String::from_utf8(buffer.clone())?.parse::<usize>()?;

        let mut content = Vec::new();
        reader.read_to_end(&mut content)?;

        if content.len() != size {
            return Err(anyhow!(
                "Incorrect content length, expected {} but was {}",
                size,
                content.len()
            ));
        }

        Ok(GitType{
            object_type,
            content
        })
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
