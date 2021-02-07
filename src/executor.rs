pub struct TokioExecutor;

impl iced::Executor for TokioExecutor {
    fn new() -> Result<Self, std::io::Error> {
        Ok(TokioExecutor {})
    }

    fn spawn(&self, future: impl Send + std::future::Future<Output = ()> + 'static) {
        tokio::spawn(future);
    }
}


