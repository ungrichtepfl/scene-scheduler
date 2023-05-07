use crate::scheduler::Scheduler;
use crate::structures::{Config, ThemeType};
use iced::theme::Theme;
use iced::widget::{
  button, column, container, horizontal_rule, radio, row, scrollable, text, text_input,
};
use iced::{alignment, Color, Element, Length, Sandbox};
use native_dialog::{FileDialog, MessageDialog, MessageType};
use serde::{Deserialize, Serialize};
use std::error::Error;

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

const GUI_CONFIG_FILE: &'static str = "gui_config.json";

#[derive(Serialize, Deserialize)]
pub struct GuiConfig {
  pub theme: ThemeType,
}

impl GuiConfig {
  pub fn load() -> Result<Self, Box<dyn Error>> {
    let config_file_path = GUI_CONFIG_FILE;
    if !std::path::Path::new(config_file_path).exists() {
      return Ok(Self::default());
    }
    let config_file = std::fs::File::open(config_file_path)?;

    let config = serde_json::from_reader(config_file)?;
    Ok(config)
  }

  pub fn save(&self) -> Result<(), Box<dyn Error>> {
    let config_file_path = GUI_CONFIG_FILE;
    let config_file = std::fs::File::create(config_file_path)?;
    serde_json::to_writer_pretty(config_file, self)?;
    Ok(())
  }

  pub fn default() -> Self {
    Self {
      theme: ThemeType::Light,
    }
  }
}

pub struct Gui {
  pub scheduler: Scheduler,
  pub schedule_sheet_num_opt: Option<usize>,
  pub scene_sheet_num_opt: Option<usize>,
  pub gui_config: GuiConfig,
}

impl Sandbox for Gui {
  type Message = Message;

  fn new() -> Self {
    let config = match Config::load() {
      Err(e) => {
        println!("Error: {}", e);
        Config::default()
      }
      Ok(c) => c,
    };
    let gui_config = match GuiConfig::load() {
      Err(e) => {
        println!("Error: {}", e);
        GuiConfig::default()
      }
      Ok(c) => c,
    };
    let scheduler = Scheduler { config };
    Self {
      schedule_sheet_num_opt: Some(scheduler.config.schedule_sheet_num),
      scene_sheet_num_opt: Some(scheduler.config.scene_sheet_num),
      scheduler,
      gui_config,
    }
  }

  fn title(&self) -> String {
    String::from("Scene Scheduler")
  }

  fn update(&mut self, message: Message) {
    match message {
      Message::CloseProgram => {
        let res_config = self.scheduler.config.save();
        if res_config.is_err() {
          println!("Error: {}", res_config.err().unwrap());
        }
        let res_gui_config = self.gui_config.save();
        if res_gui_config.is_err() {
          println!("Error: {}", res_gui_config.err().unwrap());
        }
        std::process::exit(0);
      }
      Message::ThemeChanged(theme) => self.gui_config.theme = theme,
      Message::ExcelPathChanged(value) => self.scheduler.config.excel_file_path = value,
      Message::OutDirChanged(value) => self.scheduler.config.out_dir = value,
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
          Some(path) => self.scheduler.config.excel_file_path = path.to_string_lossy().to_string(),
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
          Some(path) => self.scheduler.config.out_dir = path.to_string_lossy().to_string(),
          None => return,
        }
      }
      Message::RunProgram => {
        let res = self.scheduler.process();
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
          // Empty input: leave the config as it is.
          self.schedule_sheet_num_opt = None;
          return;
        }
        match value.parse::<usize>() {
          Err(err) => {
            println!("Could not parse ScheduleSheetNum: {}", err);
            _ = MessageDialog::new()
              .set_type(MessageType::Error)
              .set_title("Error")
              .set_text(&format!(
                "Wrong schedule sheet number entry. Must be a positiv integer but found: {}",
                value
              ))
              .show_alert();
          }
          Ok(value_usize) => {
            self.scheduler.config.schedule_sheet_num = value_usize;
            self.schedule_sheet_num_opt = Some(value_usize);
          }
        }
      }
      Message::SceneSheetNumChanged(value) => {
        if value.is_empty() {
          // Empty input: leave the config as it is.
          self.scene_sheet_num_opt = None;
          return;
        }
        match value.parse::<usize>() {
          Err(err) => {
            println!("Could not parse ScheduleSheetNum: {}", err);
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
          Ok(value_usize) => {
            self.scheduler.config.scene_sheet_num = value_usize;
            self.scene_sheet_num_opt = Some(value_usize)
          }
        }
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
          if self.gui_config.theme == *theme {
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
      &self.scheduler.config.excel_file_path,
      Message::ExcelPathChanged,
    )
    .padding(10)
    .size(20);

    let out_dir_input = text_input(
      "ICS files out directory.",
      &self.scheduler.config.out_dir,
      Message::OutDirChanged,
    )
    .padding(10)
    .size(20);

    let scene_sheet_num_label = text("Scene info sheet number:")
      .width(Length::Fill)
      .size(15)
      .style(Color::from([0.5, 0.5, 0.5]))
      .horizontal_alignment(alignment::Horizontal::Left);
    let scene_sheet_num_input_value = match self.scene_sheet_num_opt {
      Some(num) => num.to_string(),
      None => "".to_string(),
    };
    let scene_sheet_num_input = text_input(
      "Schedule sheet number",
      &scene_sheet_num_input_value,
      Message::SceneSheetNumChanged,
    )
    .padding(10)
    .size(20);

    let schedule_sheet_num_label = text("Schedule sheet number:")
      .width(Length::Fill)
      .size(15)
      .style(Color::from([0.5, 0.5, 0.5]))
      .horizontal_alignment(alignment::Horizontal::Left);

    let schedule_sheet_num_input_value = match self.schedule_sheet_num_opt {
      Some(num) => num.to_string(),
      None => "".to_string(),
    };
    let schedule_sheet_num_input = text_input(
      "Schedule info sheet number",
      &schedule_sheet_num_input_value,
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
    match self.gui_config.theme {
      ThemeType::Light => Theme::Light,
      ThemeType::Dark => Theme::Dark,
    }
  }
}
