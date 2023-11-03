//! A DAT file parser for the DAT format used by the retro community.
//! This library helps loading and searching DAT files.
use std::path::Path;

pub mod dat;
pub mod error;
pub mod optimize;

pub use dat::*;
pub use error::*;
pub use optimize::*;

pub fn read_file(path: impl AsRef<Path>) -> Result<dat::Datafile, error::Error> {
    let path = path.as_ref();
    let file = std::fs::File::open(path)?;
    from_reader(file)
}

pub fn from_reader(reader: impl std::io::Read) -> Result<dat::Datafile, error::Error> {
    let reader = std::io::BufReader::new(reader);
    let datfile = dat::Datafile::parse(reader)?;
    Ok(datfile)
}

pub fn to_writer(
    mut writer: impl std::fmt::Write,
    dat: &dat::Datafile,
) -> Result<(), error::Error> {
    quick_xml::se::to_writer_with_root(&mut writer, "datafile", dat)?;
    Ok(())
}

#[test]
fn simple() {
    let x = r#"
        <datafile>
        <header>
            <name>test</name>
            <version>1.0</version>
            <author>a</author>
            <description>d</description>
        </header>

        <game name="Test Game">
            <description>Test Game Description</description>
            <rom name="test.rom" size="123" crc="456" md5="789" sha1="012" />
        </game>
        </datafile>"#;

    let dat: dat::Datafile = from_reader(x.as_bytes()).unwrap();
    assert!(!dat.debug);
    assert_eq!(dat.header.unwrap().name, "test");
}
