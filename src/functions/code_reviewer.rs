use crate::gpt::client::Message;

use super::{common::change_request_to_file_content, GptFunction};
#[derive(Debug, Clone)]
pub struct CodeReviewer {
    prefix: &'static str,
}

impl CodeReviewer {
    const PREFIX: &'static str = "以下のコードを日本語でレビューしてください";
    pub fn new(prefix: &'static str) -> Self {
        Self { prefix }
    }
}

impl Default for CodeReviewer {
    fn default() -> Self {
        Self::new(Self::PREFIX)
    }
}
impl GptFunction for CodeReviewer {
    fn change_request(&self, request: &mut Message) {
        if let Err(err) = change_request_to_file_content(self.prefix, request) {
            eprintln!("{}", err);
        };
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        functions::common::test_tool::TestFileFactory,
        gpt::client::{Message, Role},
    };

    use super::*;
    #[test]
    #[ignore]
    fn messageの入力がfile_pathであればcode_reviewerはmessageの内容をコードレビュー依頼に変換する()
    {
        let test_file = TestFileFactory::create("tmp");
        let file_content = "fn main() { println!(\"Hello, world!\"); }";
        test_file.create_file_under_root("test.rs", file_content);

        let prefix = "以下のコードをレビューしてください";
        let mut code_reviewer = CodeReviewer::new(prefix);
        let mut message = Message::new(Role::User, "tmp/test.rs");
        code_reviewer.change_request(&mut message);

        test_file.remove_dir_all();
        assert_eq!(
            message,
            Message::new(Role::User, format!("{}\n{}", prefix, file_content))
        );
    }
}
