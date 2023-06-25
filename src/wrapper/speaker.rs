use std::process::Command;

use crate::{
    gpt::{GptClient, GptClientError},
    repl::GptMessageHandler,
};

#[cfg(target_os = "macos")]
#[derive(Debug, Clone)]
pub struct Speaker {
    model: crate::gpt::OpenAIModel,
    client: GptClient,
    default: MacSayCommandSpeaker,
}

impl GptMessageHandler<GptClientError> for Speaker {
    fn handle<F>(&mut self, message: &str, f: &F) -> Result<(), GptClientError>
    where
        F: Fn(&str),
    {
        self.say(message, f)
    }
}

impl Speaker {
    pub fn from_env() -> Result<Self, crate::gpt::GptClientError> {
        let client = GptClient::from_env()?;
        let model = crate::gpt::OpenAIModel::Gpt3Dot5Turbo;
        let default = MacSayCommandSpeaker::from_env();
        Ok(Self {
            client,
            model,
            default,
        })
    }
    pub fn say<F: Fn(&str)>(
        &mut self,
        message: &str,
        f: &F,
    ) -> Result<(), crate::gpt::GptClientError> {
        let result = self
            .client
            .chat(self.model, crate::gpt::Role::System, message, f)?;
        let result = Command::new("say")
            .args(["-v", self.default.to_name(), result.as_str()].iter())
            .output()
            .unwrap();
        if !result.status.success() {
            panic!("failed to execute process: {}", result.status);
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum MacSayCommandSpeaker {
    Karen,
    Tessa,
    Kyoko,
}

impl MacSayCommandSpeaker {
    fn from_env() -> Self {
        match std::env::var("MAC_SAY_COMMAND_SPEAKER") {
            Ok(speaker) => match speaker.as_str() {
                "Karen" => Self::Karen,
                "Tessa" => Self::Tessa,
                "Kyoko" => Self::Kyoko,
                _ => Self::Karen,
            },
            Err(_) => Self::Karen,
        }
    }
    fn to_name(&self) -> &'static str {
        match self {
            Self::Karen => "Karen",
            Self::Tessa => "Tessa",
            Self::Kyoko => "Kyoko",
        }
    }
}
