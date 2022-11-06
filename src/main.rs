use calamine::{open_workbook, Error, Reader, Xlsx};
use std::path::Path;

fn read_excel(path: &str, sheet: &str) -> Result<(), Error> {
    let mut workbook: Xlsx<_> = open_workbook(path)?;
    let range = workbook
        .worksheet_range(sheet)
        .ok_or(Error::Msg("Cannot find sheet."))??;

    for row in range.rows() {
        println!("row={:?}, row[0]={:?}", row, row[0]);
    }
    Ok(())
}

fn main() {
    let root_dir = Path::new(file!())
        .parent()
        .and_then(|p| p.parent())
        .expect("Root file path not found. File has probably moved.");
    let test_file_path = root_dir.join("tests/data/test_scedule.xlsx");
    let sheet_name = "Sheet1";
    if let Err(e) = read_excel(
        test_file_path
            .to_str()
            .expect("Check file name, wrong UTF-8 encoding for this OS."),
        sheet_name,
    ) {
        println!("Error while reading excel: {e}");
        std::process::exit(1);
    }
}
