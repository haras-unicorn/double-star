use iced::{widget::text, Element, Task};

pub fn run() -> anyhow::Result<()> {
  Ok(
    iced::application::application(
      Orbitus::title,
      Orbitus::update,
      Orbitus::view,
    )
    .run_with(Orbitus::new)?,
  )
}

#[derive(Debug, Clone)]
enum Message {}

struct Orbitus {}

impl Orbitus {
  fn new() -> (Self, Task<Message>) {
    (Self {}, Task::none())
  }

  fn title(&self) -> String {
    String::from("Orbitus")
  }

  fn update(&mut self, message: Message) -> Task<Message> {
    match message {}
  }

  fn view(&self) -> Element<Message> {
    text("Hello World!").into()
  }
}
