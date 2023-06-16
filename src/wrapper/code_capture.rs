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
    pub fn code(&self) -> Option<Code> {
        let mut chars = self.inner.chars().skip_while(|c| c != &'`');
        let (Some('`'), Some('`'),Some('`')) = (chars.next(),chars.next(), chars.next()) else {
            return None;
        };
        // not reach code
        if !chars.clone().any(|c| c == '\n') {
            return None;
        }
        let lang = chars
            .by_ref()
            .take_while(|c| c != &'\n')
            .collect::<String>();
        let code = chars.collect::<String>();
        let code = code.split("```").next()?;
        Some(Code {
            code: code.to_string(),
            lang: Lang::from_str(lang.as_str())?,
        })
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
}
impl Lang {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "rust" => Some(Self::Rust),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_code_capture() {
        let mut sut = CodeCapture::new();
        let line = "以下のコードは，1から10までの整数の和を求めるプログラムです。";
        sut.add(line);
        let before_code = "`";
        sut.add(before_code);
        sut.add(before_code);
        let code = "`ru";
        sut.add(code);
        let code = "st";
        sut.add(code);
        let code = "\n";
        sut.add(code);
        let code = "print";
        sut.add(code);
        let code = "ln!(\"Hello, world!\");\n";
        sut.add(code);
        let code = "```";
        sut.add(code);
        assert_eq!(
            sut.code(),
            Some(Code {
                code: "println!(\"Hello, world!\");\n".to_string(),
                lang: Lang::Rust
            })
        );
    }
}
