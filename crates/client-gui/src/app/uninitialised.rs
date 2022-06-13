use crate::{App, Message};

use iced::{Command, Element, Subscription};

pub(crate) struct Data(pub(crate) State);

pub(crate) enum State {
    Welcome,
    Secret,
    Pair,
    Continue,
}

impl Data {
    pub(crate) fn update(self, _message: Message) -> (App, Command<Message>) {
        todo!();
    }

    pub(crate) fn view(self) -> (App, Element<'static, Message>) {
        todo!();
    }

    pub(crate) fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}
