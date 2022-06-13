use crate::{
    app::{initialised, uninitialised},
    App, Loaded, Message,
};

use std::sync::Arc;

use iced::{Command, Element, Subscription};
use memorage_core::Mutex;

pub(crate) struct Data;

impl Data {
    pub(crate) fn update(self, message: Message) -> (App, Command<Message>) {
        (
            match message {
                Message::Loaded(Loaded::Uninitialised) => {
                    App::Uninitialised(uninitialised::Data(uninitialised::State::Welcome))
                }
                Message::Loaded(Loaded::Initialised { config, data }) => {
                    App::Initialised(initialised::Data {
                        config: Arc::new(Mutex::new(config)),
                        data: Arc::new(Mutex::new(data)),
                        state: initialised::State::Main,
                    })
                }
                _ => App::Loading(self),
            },
            Command::none(),
        )
    }

    pub(crate) fn view(self) -> (App, Element<'static, Message>) {
        todo!();
    }

    pub(crate) fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}
