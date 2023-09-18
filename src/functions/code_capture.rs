use super::GptFunction;

//use std::{cell::RefCell, io::Write};
//
//use rand::Rng;
//
//use crate::{
//    gpt::{GptClient, GptClientError, OpenAIModel, Role},
//    repl::GptMessageHandler,
//};
//
//pub struct SampleFileMaker {
//    rand: rand::rngs::ThreadRng,
//}
//
//impl SampleFileMaker {
//    const PREFIX: &'static str = "sample_for_gpt_";
//    pub fn new() -> Self {
//        Self {
//            rand: rand::thread_rng(),
//        }
//    }
//    fn make_filename(&mut self) -> String {
//        let mut file_name = String::from(Self::PREFIX);
//        for _ in 0..6 {
//            file_name.push((self.rand.gen_range(0..26) + 97) as u8 as char);
//        }
//        file_name
//    }
//}
//
//impl CodeWriter for SampleFileMaker {
//    fn write_all(&mut self, code: Code) -> Result<(), std::io::Error> {
//        let filename = match code.extends_str() {
//            Some(ex) => format!("{}.{}", self.make_filename(), ex),
//            None => self.make_filename(),
//        };
//        let mut file = std::fs::File::create(filename)?;
//        file.write_all(code.as_bytes())?;
//        Ok(())
//    }
//}
//
pub trait CodeWriter {
    fn write_all(&mut self, code: Vec<Code>) -> Result<(), std::io::Error>;
}
//
//#[derive(Debug, Clone)]
//pub struct CodeCaptureGpt<W: CodeWriter> {
//    gpt: GptClient,
//    code_capture: RefCell<CodeCapture>,
//    w: W,
//}
//
//impl<W: CodeWriter> GptMessageHandler<GptClientError> for CodeCaptureGpt<W> {
//    fn clear_history(&mut self) {
//        self.gpt.clear_history();
//    }
//    fn handle<F>(&mut self, message: &str, f: &F) -> Result<(), GptClientError>
//    where
//        F: Fn(&str) -> (),
//    {
//        let result = self
//            .gpt
//            .chat(OpenAIModel::Gpt3Dot5Turbo, Role::User, message, &|event| {
//                f(event);
//            })?;
//        self.code_capture.borrow_mut().add(&result);
//        let codes = self.code_capture.borrow().get_codes();
//        codes.into_iter().for_each(|code| {
//            self.w.write_all(code).unwrap();
//        });
//        Ok(())
//    }
//}
//
//impl<W: CodeWriter> CodeCaptureGpt<W> {
//    pub fn from_env(w: W) -> Result<Self, GptClientError> {
//        let mut gpt = GptClient::from_env()?;
//        gpt.chat(OpenAIModel::Gpt3Dot5Turbo, Role::System, "私がお願いするプログラミングの記述に対するレスポンスは全て```プログラミング言語名で初めて表現してください", &|_| {})?;
//        // set first command for system
//        Ok(Self {
//            gpt,
//            code_capture: RefCell::new(CodeCapture::new()),
//            w,
//        })
//    }
//}
//
//
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
    fn action_at_end(&mut self) {
        self.writer.write_all(self.inner.get_codes()).unwrap();
    }
}
#[derive(Debug, Clone)]
pub struct CodeCapture {
    inner: String,
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
                let mut lang_and_code = line.splitn(2, "\n");
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

#[cfg(test)]
mod tests {
    use crate::gpt::client::{ChatResponse, HandleResult};

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
    fn gptのレスポンス終了時にcodeが存在していればwriterを利用して書き込みを行う() {
        let mut buf = String::new();
        let mut function = GptCodeCapture::new(&mut buf);
        let code = "fn main(){println!();}";
        function.handle_stream(&ChatResponse::DeltaContent("```rust\n".to_string()));
        function.handle_stream(&ChatResponse::DeltaContent(code.to_string()));
        function.handle_stream(&ChatResponse::DeltaContent("```".to_string()));
        function.handle_stream(&ChatResponse::Done);

        function.action_at_end();

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
