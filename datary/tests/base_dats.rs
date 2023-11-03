use pretty_assertions::assert_eq;
use rstest::rstest;
use std::path::PathBuf;

#[rstest]
fn verify_dat(#[files("tests/okay/*")] dat: PathBuf) {
    let datfile = datary::read_file(&dat).unwrap();
    let mut output = String::new();
    datary::to_writer(&mut output, &datfile).unwrap();
    let datfile2 = datary::read_file(&dat).unwrap();

    assert_eq!(datfile, datfile2);

    // Build an optimized version.
    datfile.optimize();
}
