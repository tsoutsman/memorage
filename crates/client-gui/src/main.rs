#![deny(
    non_ascii_idents,
    // missing_docs,
    rust_2018_idioms,
    // rust_2021_compatibility,
    future_incompatible,
    missing_debug_implementations,
    missing_copy_implementations,
    nonstandard_style,
    unreachable_pub,
    clippy::missing_panics_doc,
    clippy::missing_safety_doc,
    rustdoc::broken_intra_doc_links
)]
#![allow(dead_code, clippy::large_enum_variant)]

mod app;
mod subscription;

use iced::{Command, Element, Subscription};
use memorage_client::persistent::{config::Config, data::Data};

pub(crate) enum App {
    Loading(app::loading::Data),
    Uninitialised(app::uninitialised::Data),
    Initialised(app::initialised::Data),
}

impl iced::Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        // (Self::Loading, Command::perform(todo!(), todo!()))
        todo!();
    }

    fn title(&self) -> String {
        String::from("Memorage")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        use std::{panic, ptr};

        // A reimplementation of take_mut that allows returning both app and element.
        let old_t = unsafe { ptr::read(self) };
        let (app, command) = panic::catch_unwind(panic::AssertUnwindSafe(|| match old_t {
            App::Loading(data) => data.update(message),
            App::Uninitialised(data) => data.update(message),
            App::Initialised(data) => data.update(message),
        }))
        .unwrap_or_else(|_| ::std::process::abort());
        unsafe { ptr::write(self, app) };
        command
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        use std::{panic, ptr};

        // A reimplementation of take_mut that allows returning both app and element.
        let old_t = unsafe { ptr::read(self) };
        let (app, element) = panic::catch_unwind(panic::AssertUnwindSafe(|| match old_t {
            App::Loading(data) => data.view(),
            App::Uninitialised(data) => data.view(),
            App::Initialised(data) => data.view(),
        }))
        .unwrap_or_else(|_| ::std::process::abort());
        unsafe { ptr::write(self, app) };
        element
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        match self {
            App::Loading(data) => data.subscription(),
            App::Uninitialised(data) => data.subscription(),
            App::Initialised(data) => data.subscription(),
        }
    }
}

#[derive(Debug)]
enum Message {
    Loaded(Loaded),
    Incoming(subscription::incoming::Event),
}

#[derive(Debug)]
enum Loaded {
    Uninitialised,
    Initialised { config: Config, data: Data },
}

fn main() -> iced::Result {
    <App as iced::Application>::run(iced::Settings::default())
}
