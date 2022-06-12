mod incoming;

use std::sync::Arc;

use iced::{Command, Element, Subscription, Text};
use memorage_client::persistent::{config::Config, data::Data};
use memorage_core::Mutex;

struct App {
    config: Arc<Mutex<Config>>,
    data: Arc<Mutex<Data>>,
}

impl iced::Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        #[allow(unreachable_code)]
        (
            Self {
                config: todo!(),
                data: todo!(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Memorage")
    }

    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        Text::new("Hello, world!").into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        incoming::handle(self.data.clone(), self.config.clone()).map(Message::Incoming)
    }
}

#[derive(Debug)]
enum Message {
    Incoming(incoming::Event),
}

fn main() -> iced::Result {
    <App as iced::Application>::run(iced::Settings::default())
}
