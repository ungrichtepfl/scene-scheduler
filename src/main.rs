use calamine::{open_workbook, Error, Reader, Xlsx};

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
    let path = std::path::Path::new(file!())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests/data/test_scedule.xlsx");
    read_excel(path.to_str().unwrap(), "Sheet1").unwrap();
}
