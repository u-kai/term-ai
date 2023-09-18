use std::{fs::File, io::Read, path::Path};

use crate::gpt::client::Message;

pub fn is_file_path(path: &str) -> bool {
    let path = path.trim();
    let path = Path::new(path);
    path.exists() && path.is_file()
}
pub fn change_request_to_file_content(
    prefix: &str,
    request: &mut Message,
) -> Result<(), std::io::Error> {
    if is_file_path(&request.content) {
        let path = request.content.trim();
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let message_content = request.change_content();
        *message_content = format!("{}\n{}", prefix, content);
    };
    Ok(())
}

#[cfg(test)]

pub mod test_tool {
    use std::io::Write;

    pub struct TestFileFactory {
        root: String,
    }

    impl TestFileFactory {
        pub fn create(root: &str) -> Self {
            let this = Self {
                root: root.to_string(),
            };
            this.remove_dir_all();
            this.create_root();
            this
        }
        pub fn create_file_under_root(&self, child_path: &str, content: &str) {
            let path = format!("{}/{}", self.root, child_path);
            let mut file = std::fs::File::create(path).unwrap();
            file.write_all(content.as_bytes()).unwrap();
        }
        pub fn create_root(&self) {
            std::fs::create_dir(self.root.as_str()).unwrap();
        }
        pub fn remove_dir_all(&self) {
            std::fs::remove_dir_all(self.root.as_str()).unwrap_or_default();
        }
    }
}
