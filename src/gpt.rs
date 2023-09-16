fn proxy_from_env() -> Option<String> {
    match std::env::var("HTTPS_PROXY") {
        Ok(proxy) => Some(proxy),
        Err(_) => match std::env::var("https_proxy") {
            Ok(proxy) => Some(proxy),
            Err(_) => match std::env::var("HTTP_PROXY") {
                Ok(proxy) => Some(proxy),
                Err(_) => match std::env::var("http_proxy") {
                    Ok(proxy) => Some(proxy),
                    Err(_) => None,
                },
            },
        },
    }
}
pub mod chat;
pub mod client;
