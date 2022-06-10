use iced::{Command, Element, Text};

struct App;

impl iced::Application for App {
    type Executor = iced::executor::Default;
    type Message = ();
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (Self, Command::none())
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
}

fn main() -> iced::Result {
    <App as iced::Application>::run(iced::Settings::default())
}
