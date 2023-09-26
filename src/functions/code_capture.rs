use std::io::Write;

use super::GptFunction;

use rand::Rng;
pub struct SampleFileWriter<R: RandGenerator> {
    root_dir: String,
    rand: R,
}

impl<R: RandGenerator> SampleFileWriter<R> {
    const PREFIX: &'static str = "sample_for_gpt_";
    pub fn new(root_dir: &str, rand: R) -> Self {
        Self {
            root_dir: root_dir.to_string(),
            rand,
        }
    }
    fn make_filepath(&mut self, code: &Code) -> String {
        format!(
            "{}/{}{}.{}",
            self.root_dir,
            Self::PREFIX,
            self.rand.gen(),
            code.extends_str().unwrap_or("txt")
        )
    }
}

impl<R: RandGenerator> CodeWriter for SampleFileWriter<R> {
    fn write_all(&mut self, codes: Vec<Code>) -> Result<(), std::io::Error> {
        codes
            .iter()
            .map(|code| {
                let filepath = self.make_filepath(code);
                let mut file = std::fs::File::create(filepath)?;
                file.write_all(code.as_bytes())
            })
            .fold(Ok(()), |acc, result| match acc {
                Ok(_) => result,
                Err(e) => Err(e),
            })
    }
}

pub trait CodeWriter {
    fn write_all(&mut self, code: Vec<Code>) -> Result<(), std::io::Error>;
}
#[derive(Debug, Clone)]
pub struct GptCodeCapture<W: CodeWriter> {
    writer: W,
    inner: CodeCapture,
}
impl<W: CodeWriter> GptCodeCapture<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            inner: CodeCapture::new(),
        }
    }
    pub fn get_codes(&self) -> Vec<Code> {
        self.inner.get_codes()
    }
}

impl GptCodeCapture<SampleFileWriter<DefaultRandGenerator>> {
    pub fn new_with_file_writer(root_dir: &str) -> Self {
        Self::new(SampleFileWriter::new(root_dir, DefaultRandGenerator::new()))
    }
}
impl<W: CodeWriter> GptFunction for GptCodeCapture<W> {
    fn handle_stream(
        &mut self,
        response: &crate::gpt::client::ChatResponse,
    ) -> crate::gpt::client::HandleResult {
        match response {
            crate::gpt::client::ChatResponse::DeltaContent(content) => {
                self.inner.add(content);
                crate::gpt::client::HandleResult::Progress
            }
            crate::gpt::client::ChatResponse::Done => crate::gpt::client::HandleResult::Done,
        }
    }
    fn action_at_end(&mut self) -> Result<(), Box<dyn std::error::Error + 'static>> {
        self.writer
            .write_all(self.inner.get_codes())
            .map_err(|e| e.into())
    }
}
#[derive(Debug, Clone)]
pub struct CodeCapture {
    inner: String,
}
impl Default for CodeCapture {
    fn default() -> Self {
        Self::new()
    }
}
impl CodeCapture {
    pub fn new() -> Self {
        Self {
            inner: String::new(),
        }
    }
    pub fn add(&mut self, line: &str) {
        self.inner.push_str(line);
    }
    pub fn get_codes(&self) -> Vec<Code> {
        // init words
        // lang and code
        // empty or other
        // lang and code
        // ...
        // empty or other
        self.inner
            .split("```")
            .enumerate()
            .filter_map(|(i, line)| {
                if i % 2 == 0 {
                    return None;
                }
                let mut lang_and_code = line.splitn(2, '\n');
                let Some(lang) = lang_and_code.next() else {
                    // case output in progress
                    // then None
                    return None;
                };
                match lang_and_code.next() {
                    // case code output is not yet
                    Some("") | None => None,
                    Some(code) => Some(Code {
                        code: code.to_string(),
                        lang: Lang::from_str(lang),
                    }),
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Code {
    code: String,
    lang: Lang,
}

impl Code {
    pub fn extends_str(&self) -> Option<&str> {
        match self.lang {
            Lang::Unknown => None,
            _ => Some(self.lang.to_extend()),
        }
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.code.as_bytes()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lang {
    Rust,
    Python,
    Go,
    Java,
    JavaScript,
    TypeScript,
    Ruby,
    Bash,
    Haskell,
    Yaml,
    Json,
    Unknown,
}
impl Lang {
    fn to_extend(&self) -> &str {
        match self {
            Self::Rust => "rs",
            Self::Python => "py",
            Self::Go => "go",
            Self::Java => "java",
            Self::Haskell => "hs",
            Self::JavaScript => "js",
            Self::TypeScript => "ts",
            Self::Ruby => "rb",
            Self::Bash => "sh",
            Self::Yaml => "yaml",
            Self::Json => "json",
            Self::Unknown => "",
        }
    }
    #[allow(dead_code)]
    fn to_str(&self) -> &str {
        match self {
            Self::Rust => "rust",
            Self::Python => "python",
            Self::Go => "go",
            Self::Java => "java",
            Self::Haskell => "haskell",
            Self::JavaScript => "javascript",
            Self::TypeScript => "typescript",
            Self::Ruby => "ruby",
            Self::Bash => "bash",
            Self::Yaml => "yaml",
            Self::Json => "json",
            Self::Unknown => "",
        }
    }
    fn from_str(s: &str) -> Self {
        match s {
            "rust" => Self::Rust,
            "python" => Self::Python,
            "go" => Self::Go,
            "haskell" => Self::Haskell,
            "java" => Self::Java,
            "javascript" => Self::JavaScript,
            "typescript" => Self::TypeScript,
            "ruby" => Self::Ruby,
            "sh" => Self::Bash,
            "yaml" => Self::Yaml,
            "json" => Self::Json,
            _ => Self::Unknown,
        }
    }
}

pub trait RandGenerator {
    fn gen(&mut self) -> usize;
}
pub struct DefaultRandGenerator {
    rand: rand::rngs::ThreadRng,
}
impl DefaultRandGenerator {
    pub fn new() -> Self {
        Self {
            rand: rand::thread_rng(),
        }
    }
}
impl Default for DefaultRandGenerator {
    fn default() -> Self {
        Self::new()
    }
}
impl RandGenerator for DefaultRandGenerator {
    fn gen(&mut self) -> usize {
        self.rand.gen()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        functions::common::test_tool::TestFileFactory,
        gpt::client::{ChatResponse, HandleResult},
    };

    impl CodeWriter for &mut String {
        fn write_all(&mut self, codes: Vec<Code>) -> Result<(), std::io::Error> {
            for code in codes {
                self.push_str(&code.code);
            }
            Ok(())
        }
    }
    use super::*;
    #[test]
    #[ignore]
    fn sample_file_writerはランダムな名前で指定ディレクトリにファイルを作成する() {
        struct FakeRand {
            rand: usize,
        }
        impl RandGenerator for FakeRand {
            fn gen(&mut self) -> usize {
                self.rand
            }
        }
        let root_dir = "tmp";
        let rand = 0;
        let test_file = TestFileFactory::create(root_dir);
        let file_writer = SampleFileWriter::new(root_dir, FakeRand { rand });

        let mut function = GptCodeCapture::new(file_writer);
        let code = "fn main(){println!();}";
        function.handle_stream(&ChatResponse::DeltaContent("```rust\n".to_string()));
        function.handle_stream(&ChatResponse::DeltaContent(code.to_string()));
        function.handle_stream(&ChatResponse::DeltaContent("```".to_string()));
        function.handle_stream(&ChatResponse::Done);

        function.action_at_end().unwrap();

        let result = std::fs::read_to_string(format!(
            "{}/{}{}.rs",
            root_dir,
            SampleFileWriter::<FakeRand>::PREFIX,
            rand
        ))
        .unwrap();

        test_file.remove_dir_all();
        assert_eq!(result, code);
    }
    #[test]
    fn gptのレスポンス終了時にcodeが存在していればwriterを利用して書き込みを行う() {
        let mut buf = String::new();
        let mut function = GptCodeCapture::new(&mut buf);
        let code = "fn main(){println!();}";
        function.handle_stream(&ChatResponse::DeltaContent("```rust\n".to_string()));
        function.handle_stream(&ChatResponse::DeltaContent(code.to_string()));
        function.handle_stream(&ChatResponse::DeltaContent("```".to_string()));
        function.handle_stream(&ChatResponse::Done);

        function.action_at_end().unwrap();

        assert_eq!(buf, code);
    }
    #[test]
    fn gptからのsseレスポンスを受け取って内部に保存する() {
        let mut buf = String::new();
        let mut function = GptCodeCapture::new(&mut buf);
        let code = "fn main(){println!();}";
        let progress = function.handle_stream(&ChatResponse::DeltaContent("```rust\n".to_string()));
        assert_eq!(progress, HandleResult::Progress);

        let progress = function.handle_stream(&ChatResponse::DeltaContent(code.to_string()));
        assert_eq!(progress, HandleResult::Progress);

        let progress = function.handle_stream(&ChatResponse::DeltaContent("```".to_string()));
        assert_eq!(progress, HandleResult::Progress);

        let progress = function.handle_stream(&ChatResponse::Done);
        assert_eq!(progress, HandleResult::Done);

        assert_eq!(
            function.get_codes(),
            vec![Code {
                code: code.to_string(),
                lang: Lang::Rust,
            }]
        );
    }
    #[test]
    fn 複数のコードに対しても動作する() {
        let mut sut = CodeCapture::new();
        let line = "以下のコードは，1から10までの整数の和を求めるプログラムです。";
        sut.add(line);
        let before_code = "`";
        sut.add(before_code);
        sut.add(before_code);
        let code = "`";
        sut.add(code);
        let code = "`\n";
        sut.add(code);
        let code = "print";
        sut.add(code);
        let code = "ln!(\"Hello, world!\");\n";
        sut.add(code);
        let code = "```\n";
        sut.add(code);
        assert_eq!(
            sut.get_codes(),
            vec![Code {
                code: "println!(\"Hello, world!\");\n".to_string(),
                lang: Lang::Unknown,
            }]
        );
        let line = "出力以下です\n";
        sut.add(line);
        let before_code = "`";
        sut.add(before_code);
        sut.add(before_code);
        let code = "`";
        sut.add(code);
        let code = "`\n";
        sut.add(code);
        let code = "Hello,";
        assert_eq!(
            sut.get_codes(),
            vec![Code {
                code: "println!(\"Hello, world!\");\n".to_string(),
                lang: Lang::Unknown,
            }]
        );
        sut.add(code);
        let code = " world\n";
        sut.add(code);
        let code = "```";
        sut.add(code);
        assert_eq!(
            sut.get_codes(),
            vec![
                Code {
                    code: "println!(\"Hello, world!\");\n".to_string(),
                    lang: Lang::Unknown,
                },
                Code {
                    code: "Hello, world\n".to_string(),
                    lang: Lang::Unknown,
                }
            ]
        );
        let code = "\nthis code is simple code\n";
        sut.add(code);
        assert_eq!(
            sut.get_codes(),
            vec![
                Code {
                    code: "println!(\"Hello, world!\");\n".to_string(),
                    lang: Lang::Unknown,
                },
                Code {
                    code: "Hello, world\n".to_string(),
                    lang: Lang::Unknown,
                }
            ]
        );
    }
}
