//use term_ai::{
//    gpt::GptClient,
//    wrapper::{
//        any::{AnyHandler, GptInput, Printer},
//        speaker::MacSpeaker,
//        translator::FileTranslator,
//    },
//};
fn main() {}

//fn main() {
//    let mut any = AnyHandler::new(GptClient::from_env().unwrap());
//    let file_translator = FileTranslator::new();
//    let printer = Printer::new();
//    let speaker = MacSpeaker::new();
//    any.add_event_handler(Box::new(printer));
//    any.add_input_convertor(Box::new(file_translator.clone()));
//    any.add_response_handler(Box::new(file_translator));
//    any.add_response_handler(Box::new(speaker));
//    let mut message = String::new();
//    std::io::stdin().read_line(&mut message).unwrap();
//    any.handle(GptInput::new(
//        message,
//        term_ai::gpt::OpenAIModel::Gpt3Dot5Turbo,
//        term_ai::gpt::Role::User,
//    ))
//    .unwrap();
//}
