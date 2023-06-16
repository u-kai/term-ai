#[derive(Debug, Clone)]
pub struct CodeCapture {
    inner: String,
    back_quote_count: usize,
    is_next_lang: bool,
}
impl CodeCapture {
    pub fn new() -> Self {
        Self {
            back_quote_count: 0,
            is_next_lang: false,
            inner: String::new(),
        }
    }
    pub fn add(&mut self, line: &str) {
        for c in line.chars() {
            if c == '`' {
                self.back_quote_count += 1;
                if self.back_quote_count == 3 {
                    self.is_next_lang = true;
                    self.back_quote_count = 0;
                }
                continue;
            }
            if self.is_next_lang {
                self.is_next_lang = false;
                self.inner.push_str("```");
                self.inner.push(c);
                continue;
            }
        }
        self.inner.push_str(line);
    }
    pub fn code(&self) -> Option<Code> {
        let code = self.inner.split("```").nth(1)?;
        let mut code = code.split('\n');
        let lang = code.next()?.trim();
        let code = code.collect::<Vec<_>>().join("\n");
        let lang = match lang {
            "rust" => Lang::Rust,
            _ => return None,
        };
        Some(Code {
            code: code.to_string(),
            lang,
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_code_capture() {
        let mut sut = CodeCapture::new();
        let line = "以下のコードは，1から10までの整数の和を求めるプログラムです。";
        sut.add(line);
        assert_eq!(sut.code(), None);
        let before_code = "`";
        sut.add(before_code);
        sut.add(before_code);
        assert_eq!(sut.code(), None);
        let code = "`ru";
        sut.add(code);
        let code = "st";
        sut.add(code);
        let code = "\n";
        sut.add(code);
        let code = "print";
        sut.add(code);
        let code = "ln!(\"Hello, world!\");\n";
        assert_eq!(sut.code(), None);
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
