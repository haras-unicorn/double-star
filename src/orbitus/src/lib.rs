use iced::{
  widget::{column, container, text, text_input, vertical_space, Space},
  Element, Length, Task,
};

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
enum Message {
  Input(String),
  Submit,
}

struct Orbitus {
  text: String,
  input: String,
}

impl Orbitus {
  fn new() -> (Self, Task<Message>) {
    (
      Self {
        text: "Hello, world!\n".to_string(),
        input: "".to_string(),
      },
      Task::none(),
    )
  }

  fn title(&self) -> String {
    String::from("Orbitus")
  }

  fn update(&mut self, message: Message) -> Task<Message> {
    match message {
      Message::Input(input) => self.input = input,
      Message::Submit => {
        self.text += self.input.as_str();
        self.text += "\n";
        self.input.clear();
      }
    };

    Task::none()
  }

  fn view(&self) -> Element<Message> {
    let input = text_input("Type here!", self.input.as_str())
      .on_input(|input| Message::Input(input))
      .on_submit(Message::Submit);
    container(
      container(column![text(self.text.as_str()), vertical_space(), input])
        .max_width(1024)
        .align_left(Length::Fill),
    )
    .center_x(Length::Fill)
    .into()
  }
}
