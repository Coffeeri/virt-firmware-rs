use std::path::PathBuf;

pub fn write_boot_csv(csv_path: &PathBuf, csv_data: &str) -> Result<(), std::io::Error> {
    let mut data: Vec<u8> = Vec::new();
    data.push(0xff);
    data.push(0xfe);
    for c in csv_data.encode_utf16() {
        for b in c.to_le_bytes() {
            data.push(b);
        }
    }

    std::fs::write(csv_path, data)?;
    Ok(())
}
