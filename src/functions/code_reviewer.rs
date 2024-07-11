use crate::gpt::client::Message;

use super::{
    common::{get_file_content, is_file_path},
    GptFunction, UserInput,
};
#[derive(Debug, Clone)]
pub struct CodeReviewer {
    prefix: &'static str,
    action: bool,
}

impl CodeReviewer {
    const PREFIX: &'static str = "以下のコードを日本語でレビューしてください";
    pub fn new(prefix: &'static str) -> Self {
        Self {
            prefix,
            action: false,
        }
    }
}

impl Default for CodeReviewer {
    fn default() -> Self {
        Self::new(Self::PREFIX)
    }
}
impl GptFunction for CodeReviewer {
    fn input_to_messages(&self, input: super::UserInput) -> Vec<Message> {
        let content = get_file_content(input.content()).unwrap_or(input.content().to_string());
        UserInput::new(content)
            .to_messages()
            .into_iter()
            .map(|mut message| {
                let content = message.change_content();
                *content = format!("{}\n{}", self.prefix, content);
                message
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        functions::{common::test_tool::TestFileFactory, UserInput},
        gpt::client::{Message, Role},
    };

    use super::*;

    #[test]
    fn messageの入力がfile_path以外であればその内容をコードレビュー依頼に変換する() {
        let src = "this is not filepath";
        let prefix = "以下のコードを日本語でレビューしてください";
        let code_reviewer = CodeReviewer::new(prefix);
        let input = UserInput::new(src);
        let messages = code_reviewer.input_to_messages(input);

        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0],
            Message::new(Role::User, format!("{}\n{}", prefix, src))
        );
    }
    #[test]
    #[ignore]
    fn messageの入力がfile_pathであればcode_reviewerはmessageの内容をコードレビュー依頼に変換する()
    {
        let test_file = TestFileFactory::create("tmp");
        let file_content = "fn main() { println!(\"Hello, world!\"); }";
        test_file.create_file_under_root("test.rs", file_content);

        let prefix = "以下のコードをレビューしてください";
        let mut code_reviewer = CodeReviewer::new(prefix);
        let input = UserInput::new("tmp/test.rs");

        code_reviewer.setup_for_action(&input);
        let messages = code_reviewer.input_to_messages(input);

        test_file.remove_dir_all();

        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0],
            Message::new(Role::User, format!("{}\n{}", prefix, file_content))
        );
    }
}
