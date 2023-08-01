pub mod parsing;

pub mod excel {
  use std::io::Error;
  use std::io::ErrorKind;

  use calamine::{open_workbook, DataType, Range, Reader, Xlsx};

  pub fn read_excel(
    path: &str,
    sheet_num: usize,
  ) -> Result<(Range<DataType>, String), calamine::Error> {
    let mut workbook: Xlsx<_> = open_workbook(path)?;
    let sheet_name = workbook
      .sheet_names()
      .get(sheet_num)
      .ok_or(calamine::Error::Io(Error::new(
        ErrorKind::NotFound,
        format!("Cannot find sheet number {} in file {}.", sheet_num, path),
      )))?
      .to_owned();
    let range = workbook
      .worksheet_range_at(sheet_num)
      .ok_or(calamine::Error::Io(Error::new(
        ErrorKind::NotFound,
        format!("Cannot find sheet number {} in file {}.", sheet_num, path),
      )))??;
    Ok((range, sheet_name))
  }
}
