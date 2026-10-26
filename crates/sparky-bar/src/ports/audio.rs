use iced::futures::future::BoxFuture;

use crate::domain::audio::AudioRequest;

pub trait AudioControl: std::fmt::Debug + Send + Sync {
    fn send(&self, request: AudioRequest) -> BoxFuture<'_, Result<(), String>>;
}
