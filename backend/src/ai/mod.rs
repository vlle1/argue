pub mod openai;

pub trait AiApi {
    type AiError;
    fn rate_limit(&self) -> u32;
    async fn request(&mut self, input: String) -> Result<String, Self::AiError>;
}
