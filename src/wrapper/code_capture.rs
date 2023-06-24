use std::{cell::RefCell, io::Write};

use crate::{
    gpt::{GptClient, GptClientError, OpenAIModel, Role},
    repl::MessageHandler,
};

#[derive(Debug, Clone)]
pub struct CodeCaptureGpt<W: Write> {
    gpt: GptClient,
    code_capture: RefCell<CodeCapture>,
    w: W,
}

impl<W: Write> MessageHandler<GptClientError> for CodeCaptureGpt<W> {
    fn handle<F>(&mut self, message: &str, f: &F) -> Result<(), GptClientError>
    where
        F: Fn(&str) -> (),
    {
        self.gpt
            .chat(OpenAIModel::Gpt3Dot5Turbo, Role::User, message, &|event| {
                self.code_capture.borrow_mut().add(event);
                f(event);
            })?;
        let codes = self.code_capture.borrow().get_codes();
        codes.into_iter().for_each(|code| {
            self.w.write_all(code.code.as_bytes()).unwrap();
            self.w.flush().unwrap();
        });
        Ok(())
    }
}

impl<W: Write> CodeCaptureGpt<W> {
    pub fn from_env(w: W) -> Result<Self, GptClientError> {
        let mut gpt = GptClient::from_env()?;
        gpt.chat(OpenAIModel::Gpt3Dot5Turbo, Role::System, "私がお願いするプログラミングの記述に対するレスポンスは全て```プログラミング言語名で初めて表現してください", &|_| {})?;
        // set first command for system
        Ok(Self {
            gpt,
            code_capture: RefCell::new(CodeCapture::new()),
            w,
        })
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
        let mut other_and_codes_other = self.inner.split("```");
        if other_and_codes_other.next().is_none() {
            return vec![];
        }
        // code,code,code
        other_and_codes_other
            .filter_map(|lang_and_codes| {
                let mut lang_and_code = lang_and_codes.splitn(2, "\n");
                let Some(lang) = lang_and_code.next() else {
                    return None;
                };
                let lang = Lang::from_str(lang);
                let code = lang_and_code.collect::<String>();
                if code == "" {
                    return None;
                }
                Some(Code { code, lang })
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Code {
    code: String,
    lang: Lang,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lang {
    Rust,
    None,
}
impl Lang {
    fn from_str(s: &str) -> Self {
        match s {
            "rust" => Self::Rust,
            _ => Self::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_code_capture_複数のコードに対しても動作する() {
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
        assert_eq!(
            sut.get_codes(),
            vec![Code {
                code: "println!(\"Hello, world!\");\n".to_string(),
                lang: Lang::None,
            }]
        );
        let code = "```";
        sut.add(code);
        assert_eq!(
            sut.get_codes(),
            vec![Code {
                code: "println!(\"Hello, world!\");\n".to_string(),
                lang: Lang::None,
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
                lang: Lang::None,
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
                    lang: Lang::None,
                },
                Code {
                    code: "Hello, world\n".to_string(),
                    lang: Lang::None,
                }
            ]
        );
    }
}
