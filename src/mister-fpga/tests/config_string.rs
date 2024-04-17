use mister_fpga::config_string::Config;
use pretty_assertions::assert_eq;
use rstest::rstest;
use std::path::PathBuf;
use std::str::FromStr;

#[rstest]
fn load_config_string(#[files("tests/assets/config_string/*")] root: PathBuf) {
    let config = std::fs::read_to_string(root.join("config")).unwrap();
    let config = Config::from_str(config.trim_end());

    assert!(config.is_ok(), "{:?}", config);

    let config = config.unwrap();
    if root.join("status_bit_mask").exists() {
        let map = config.status_bit_map_mask();
        let data = map.debug_string(true);
        let expected = std::fs::read_to_string(root.join("status_bit_mask")).unwrap();
        assert_eq!(data, expected);
    }
}
