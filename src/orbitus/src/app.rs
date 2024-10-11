use futures::StreamExt;
use iced::{
  widget::{
    button, column, container, row, text, text::danger, text_input,
    vertical_space, scrollable
  },
  Element, Length, Subscription, Task,
};

#[derive(Debug, Clone)]
pub(crate) enum Message {
  Input(String),
  DoubleStar(gravity::DoubleStarMessage),
  Config(crate::config::Config),
  Error(String),
  Ok,
  Submit,
  ConfigSubmit,
}

pub(crate) struct Orbitus {
  double_star_tx: flume::Sender<gravity::OrbitusMessage>,
  double_star_rx: flume::Receiver<gravity::DoubleStarMessage>,
  config: crate::config::Config,
  config_tx: flume::Sender<crate::config::Config>,
  config_rx:
    flume::Receiver<gravity::config::ConfigUpdate<crate::config::Config>>,
  chat: String,
  input: String,
  error: String,
}

impl Orbitus {
  pub(crate) fn new(
    double_star_tx: flume::Sender<gravity::OrbitusMessage>,
    double_star_rx: flume::Receiver<gravity::DoubleStarMessage>,
    config: crate::config::Config,
    config_tx: flume::Sender<crate::config::Config>,
    config_rx: flume::Receiver<
      gravity::config::ConfigUpdate<crate::config::Config>,
    >,
  ) -> (Self, Task<Message>) {
    (
      Self {
        double_star_tx,
        double_star_rx,
        config,
        config_tx,
        config_rx,
        chat: "Hello, world!\n".to_string(),
        input: "".to_string(),
        error: "".to_string(),
      },
      Task::none(),
    )
  }

  pub(crate) fn title(&self) -> String {
    String::from("Orbitus")
  }

  pub(crate) fn theme(&self) -> iced::theme::Theme {
    let is_dark = dark_light::detect() == dark_light::Mode::Dark;
    if is_dark {
      iced::Theme::custom(
        "orbitus".to_string(),
        palette_to_iced_palette(&self.config.ui.palette.dark),
      )
    } else {
      iced::Theme::custom(
        "orbitus".to_string(),
        palette_to_iced_palette(&self.config.ui.palette.light),
      )
    }
  }

  pub(crate) fn subscription(&self) -> Subscription<Message> {
    let double_star_sub = Subscription::run_with_id(
      "double_star",
      self
        .double_star_rx
        .clone()
        .into_stream()
        .map(Message::DoubleStar),
    );

    let config_sub = Subscription::run_with_id(
      "config",
      self
        .config_rx
        .clone()
        .into_stream()
        .map(|result| match result.error {
          Some(err) => Message::Error(err.to_string()),
          None => Message::Config(result.config),
        }),
    );

    Subscription::batch(vec![double_star_sub, config_sub])
  }

  pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
    match message {
      Message::Input(input) => self.input = input,
      Message::Submit => {
        self.chat += self.input.as_str();
        self.chat += "\n";

        let tx = self.double_star_tx.clone();
        let mut input = String::new();
        std::mem::swap(&mut input, &mut self.input);

        return Task::perform(
          async move { tx.send_async(gravity::OrbitusMessage::Submit(input)).await },
          |result| match result {
            Ok(_) => Message::Ok,
            Err(err) => Message::Error(err.to_string()),
          },
        );
      }
      Message::DoubleStar(double_star) => match double_star {
        gravity::DoubleStarMessage::Generated(generated) => {
          self.chat += generated.as_str();
        }
        gravity::DoubleStarMessage::Break => {
          self.chat += "\n";
        },
      },
      Message::Config(config) => {
        self.config = config;
      }
      Message::Error(error) => {
        self.error = error;
      }
      Message::Ok => {}
      Message::ConfigSubmit => {
        let config = self.config.clone();
        let tx = self.config_tx.clone();
        return Task::perform(
          async move { tx.send_async(config).await },
          |result| match result {
            Ok(_) => Message::Ok,
            Err(err) => Message::Error(err.to_string()),
          },
        );
      }
    };

    Task::none()
  }

  pub(crate) fn view(&self) -> Element<Message> {
    let input = text_input("Type here!", self.input.as_str())
      .on_input(Message::Input)
      .on_submit(Message::Submit)
      .width(Length::Fill);
    let config_submit =
      button(text("Submit config")).on_press(Message::ConfigSubmit);
    let input_row = row![input, config_submit];

    let chat = scrollable(text(self.chat.as_str()));
    let error = text(self.error.as_str()).style(danger);
    let column = column![chat, vertical_space(), error, input_row];

    container(
      container(column)
        .max_width(1024)
        .align_left(Length::Fill),
    )
    .center_x(Length::Fill)
    .into()
  }
}

fn palette_to_iced_palette(
  palette: &crate::config::UiPaletteModeConfig,
) -> iced::theme::Palette {
  iced::theme::Palette {
    background: palette_to_iced_color(&palette.background),
    text: palette_to_iced_color(&palette.text),
    primary: palette_to_iced_color(&palette.primary),
    success: palette_to_iced_color(&palette.success),
    danger: palette_to_iced_color(&palette.danger),
  }
}

fn palette_to_iced_color(color: &palette::Srgba) -> iced::Color {
  iced::Color {
    r: color.red,
    g: color.green,
    b: color.blue,
    a: color.alpha,
  }
}
