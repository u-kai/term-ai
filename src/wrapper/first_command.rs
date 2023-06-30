use std::io::Write;

use crate::{
    gpt::{GptClient, GptClientError, OpenAIModel},
    repl::GptMessageHandler,
};

pub trait FirstSystemCommandInput {
    fn first_system_command(&mut self, model: OpenAIModel) -> Result<(), GptClientError>;
    fn first_system_command_with_response(&mut self) -> Result<(), GptClientError>;
}

pub struct FirstSystemCommandInputModel<T: FirstSystemCommandInput> {
    model: OpenAIModel,
    client: T,
}

impl<T: FirstSystemCommandInput> GptMessageHandler<GptClientError>
    for FirstSystemCommandInputModel<T>
{
    fn handle<F>(&mut self, message: &str, f: &F) -> Result<(), GptClientError>
    where
        F: Fn(&str),
    {
        self.client.first_system_command(self.model)?;
        self.client.first_system_command_with_response()?;
        Ok(())
    }
}

pub struct FirstSystemCommand {
    model: OpenAIModel,
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
    pub fn from_env(command: &str) -> Result<Self, crate::gpt::GptClientError> {
        let mut client = GptClient::from_env()?;
        let model = crate::gpt::OpenAIModel::Gpt3Dot5Turbo;

        // set first command for system
        client.chat(model, crate::gpt::Role::System, command, &|_| {})?;

        Ok(Self { client, model })
    }
    pub fn with_display_first_response(command: &str) -> Result<Self, crate::gpt::GptClientError> {
        let mut client = GptClient::from_env()?;
        let model = crate::gpt::OpenAIModel::Gpt3Dot5Turbo;
        // set first command for system
        print!("response -> ");
        std::io::stdout().flush().unwrap();
        client.chat(model, crate::gpt::Role::System, command, &|message| {
            print!("{}", message);
            std::io::stdout().flush().unwrap();
        })?;
        println!();
        Ok(Self { client, model })
    }
}
