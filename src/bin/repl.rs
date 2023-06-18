use term_ai::repl::{GptRepl, StubChat};

fn main() {
    let mut chat = StubChat::new();
    chat.add("Hello, I am GPT-3. How are you?");
    chat.add("I am doing well. How are you?");
    chat.add("I am doing well. How are you?");
    let repl = GptRepl::new(chat);
    repl.repl();
}
