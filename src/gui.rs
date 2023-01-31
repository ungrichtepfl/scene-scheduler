use crate::scheduler;
use crate::structures::{Config, ThemeType};
use iced::theme::Theme;
use iced::widget::{
  button, column, container, horizontal_rule, radio, row, scrollable, text, text_input,
};
use iced::{alignment, Color, Element, Length, Sandbox};
use native_dialog::{FileDialog, MessageDialog, MessageType};

#[derive(Debug)]
pub struct Scheduler {
  config: Config,
}

#[derive(Debug, Clone)]
pub enum Message {
  ThemeChanged(ThemeType),
  ExcelPathChanged(String),
  ChooseExcelFile,
  OutDirChanged(String),
  ChooseOutDir,
  ScheduleSheetNumChanged(String),
  SceneSheetNumChanged(String),
  RunProgram,
  CloseProgram,
}

impl Sandbox for Scheduler {
  type Message = Message;

  fn new() -> Self {
    let config = Config::load();
    match config {
      Err(e) => {
        println!("Error: {}", e);
        Self {
          config: Config::default(),
        }
      }
      Ok(c) => Self { config: c },
    }
  }

  fn title(&self) -> String {
    String::from("Scene Scheduler")
  }

  fn update(&mut self, message: Message) {
    match message {
      Message::CloseProgram => {
        let res = self.config.save();
        if res.is_err() {
          println!("Error: {}", res.err().unwrap());
        }
        std::process::exit(0);
      }
      Message::ThemeChanged(theme) => self.config.theme = theme,
      Message::ExcelPathChanged(value) => self.config.excel_file_path = value,
      Message::OutDirChanged(value) => self.config.out_dir = value,
      Message::ChooseExcelFile => {
        let start_path = dirs::desktop_dir().map(|p| p.as_path().to_owned());
        let path = if let Some(start_path) = start_path {
          FileDialog::new()
            .set_location(&start_path)
            .add_filter("Excel File", &["xlsx"])
            .show_open_single_file()
        } else {
          println!("Error: Could not find desktop directory");
          FileDialog::new()
            .add_filter("Excel File", &["xlsx"])
            .show_open_single_file()
        };

        if path.is_err() {
          _ = MessageDialog::new()
            .set_type(MessageType::Error)
            .set_title("Error")
            .set_text(&format!(
              "Could not set excel file path: {}",
              path.as_ref().err().unwrap()
            ))
            .show_alert();
        }

        match path.unwrap() {
          Some(path) => self.config.excel_file_path = path.to_string_lossy().to_string(),
          None => return,
        }
      }
      Message::ChooseOutDir => {
        let start_path = dirs::desktop_dir().map(|p| p.as_path().to_owned());
        let path = if let Some(start_path) = start_path {
          FileDialog::new()
            .set_location(&start_path)
            .show_open_single_dir()
        } else {
          println!("Error: Could not find desktop directory");
          FileDialog::new().show_open_single_dir()
        };

        if path.is_err() {
          _ = MessageDialog::new()
            .set_type(MessageType::Error)
            .set_title("Error")
            .set_text(&format!(
              "Could not set out dir : {}",
              path.as_ref().err().unwrap()
            ))
            .show_alert();
        }

        match path.unwrap() {
          Some(path) => self.config.out_dir = path.to_string_lossy().to_string(),
          None => return,
        }
      }
      Message::RunProgram => {
        let res = scheduler::run(self.config.clone());
        match res {
          Ok(_) => {
            _ = MessageDialog::new()
              .set_type(MessageType::Info)
              .set_title("Success")
              .set_text("Successfully generated ics files")
              .show_alert();
          }
          Err(e) => {
            println!("Error: {}", e);
            _ = MessageDialog::new()
              .set_type(MessageType::Error)
              .set_title("Error")
              .set_text(&format!("Could not generate ics files: {}", e))
              .show_alert();
          }
        }
      }
      Message::ScheduleSheetNumChanged(value) => {
        if value.is_empty() {
          // FIXME: Introduce optional to display empty string
          return;
        }
        let value_usize = value.parse::<usize>();
        if value_usize.is_err() {
          println!(
            "Could not parse ScheduleSheetNum: {}",
            value_usize.err().unwrap()
          );
          _ = MessageDialog::new()
            .set_type(MessageType::Error)
            .set_title("Error")
            .set_text(&format!(
              "Wrong schedule sheet number entry. Must be a positiv integer but found: {}",
              value
            ))
            .show_alert();
          return;
        }
        self.config.schedule_sheet_num = value_usize.unwrap();
      }
      Message::SceneSheetNumChanged(value) => {
        if value.is_empty() {
          // FIXME: Introduce optional to display empty string
          return;
        }
        let value_usize = value.parse::<usize>();
        if value_usize.is_err() {
          println!(
            "Could not parse ScheduleSheetNum: {}",
            value_usize.err().unwrap()
          );
          _ = MessageDialog::new()
            .set_type(MessageType::Error)
            .set_title("Error")
            .set_text(&format!(
              "Wrong scene sheet number entry. Must be a positiv integer but found: {}",
              value
            ))
            .show_alert();
          return;
        }
        self.config.scene_sheet_num = value_usize.unwrap();
      }
    }
  }

  fn view(&self) -> Element<Message> {
    let choose_theme = [ThemeType::Light, ThemeType::Dark].iter().fold(
      column![text("Choose a theme:")].spacing(10),
      |column, theme| {
        column.push(radio(
          format!("{theme:?}"),
          *theme,
          if self.config.theme == *theme {
            Some(*theme)
          } else {
            None
          },
          Message::ThemeChanged,
        ))
      },
    );

    let title = text("Scene Scheduler")
      .width(Length::Fill)
      .size(30)
      .style(Color::from([0.5, 0.5, 0.5]))
      .horizontal_alignment(alignment::Horizontal::Center);

    let excel_file_path_input = text_input(
      "Excel file path",
      &self.config.excel_file_path,
      Message::ExcelPathChanged,
    )
    .padding(10)
    .size(20);

    let out_dir_input = text_input(
      "ICS files out directory.",
      &self.config.out_dir,
      Message::OutDirChanged,
    )
    .padding(10)
    .size(20);

    let scene_sheet_num_label = text("Scene info sheet number:")
      .width(Length::Fill)
      .size(15)
      .style(Color::from([0.5, 0.5, 0.5]))
      .horizontal_alignment(alignment::Horizontal::Left);

    let scene_sheet_num_input = text_input(
      "Schedule sheet number",
      &self.config.scene_sheet_num.to_string(),
      Message::SceneSheetNumChanged,
    )
    .padding(10)
    .size(20);

    let schedule_sheet_num_label = text("Schedule sheet number:")
      .width(Length::Fill)
      .size(15)
      .style(Color::from([0.5, 0.5, 0.5]))
      .horizontal_alignment(alignment::Horizontal::Left);

    let schedule_sheet_num_input = text_input(
      "Schedule info sheet number",
      &self.config.schedule_sheet_num.to_string(),
      Message::ScheduleSheetNumChanged,
    )
    .padding(10)
    .size(20);

    let generate_ics_button = button("Generate ICS Files.")
      .padding(10)
      .on_press(Message::RunProgram);

    let choose_excel_file_button = button("Choose Excel File")
      .padding(10)
      .on_press(Message::ChooseExcelFile);

    let choose_out_dir_button = button("Choose Out Dir")
      .padding(10)
      .on_press(Message::ChooseOutDir);

    let close_button = button("Close").padding(10).on_press(Message::CloseProgram);

    let content = column![
      title,
      row![excel_file_path_input, choose_excel_file_button].spacing(10),
      row![out_dir_input, choose_out_dir_button].spacing(10),
      column![
        row![schedule_sheet_num_label, scene_sheet_num_label].spacing(10),
        row![schedule_sheet_num_input, scene_sheet_num_input].spacing(10),
      ],
      generate_ics_button,
      horizontal_rule(38),
      choose_theme,
      horizontal_rule(38),
      close_button,
    ]
    .spacing(20)
    .padding(20)
    .max_width(600);
    // .align_items(Alignment::Center);

    scrollable(
      container(content)
        .width(Length::Fill)
        // .height(Length::Fill)
        .center_x(), // .center_y(),
    )
    .into()
  }

  fn theme(&self) -> Theme {
    match self.config.theme {
      ThemeType::Light => Theme::Light,
      ThemeType::Dark => Theme::Dark,
    }
  }
}
