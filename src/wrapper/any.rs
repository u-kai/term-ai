pub trait GptEventListener {
    fn on_event(&mut self, input: &str);
    fn is_trigger(&mut self, input: &str) -> bool;
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn 複数のevent_listenerを登録して実行可能() {
        //let mut sut = AnyGptHandler::new();
        //let mut listener1 = CodeCapture::new();
        //let mut listener2 = CodeReviewer::from_env().unwrap();
        //sut.add_listener(Box::new(listener1));
        //sut.add_listener(Box::new(listener2));
        //sut.on_event("message");
    }
}
