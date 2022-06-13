use crate::{App, Message};

use std::sync::Arc;

use iced::{Command, Element, Subscription};
use memorage_core::Mutex;

pub(crate) struct Data {
    pub(crate) config: Arc<Mutex<memorage_client::persistent::config::Config>>,
    pub(crate) data: Arc<Mutex<memorage_client::persistent::data::Data>>,
    pub(crate) state: State,
}

pub(crate) enum State {
    Main,
    Settings,
}

impl Data {
    pub(crate) fn update(self, _message: Message) -> (App, Command<Message>) {
        todo!();
    }

    pub(crate) fn view(self) -> (App, Element<'static, Message>) {
        todo!();
    }

    pub(crate) fn subscription(&self) -> Subscription<Message> {
        crate::subscription::incoming::handle(self.data.clone(), self.config.clone())
            .map(Message::Incoming)
    }
}
