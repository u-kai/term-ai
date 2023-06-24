use crate::{
    gpt::{GptClient, GptClientError},
    repl::GptMessageHandler,
};

pub struct FirstSystemCommand {
    model: crate::gpt::OpenAIModel,
    client: GptClient,
}

impl GptMessageHandler<GptClientError> for FirstSystemCommand {
    fn handle<F>(&mut self, message: &str, f: &F) -> Result<(), GptClientError>
    where
        F: Fn(&str),
    {
        self.client
            .chat(self.model, crate::gpt::Role::User, message, f)?;
        Ok(())
    }
}

impl FirstSystemCommand {
    pub fn from_env(command: &'static str) -> Result<Self, crate::gpt::GptClientError> {
        let mut client = GptClient::from_env()?;
        let model = crate::gpt::OpenAIModel::Gpt3Dot5Turbo;

        // set first command for system
        client.chat(model, crate::gpt::Role::System, command, &|_| {})?;

        Ok(Self { client, model })
    }
}
