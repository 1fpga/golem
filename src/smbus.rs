use std::ffi::c_int;

extern "C" {
    pub fn i2c_open(dev_address: c_int, is_smbus: c_int) -> c_int;
    pub fn i2c_close(fd: c_int);

    pub fn i2c_smbus_write_quick(file: c_int, value: u8) -> c_int;
    pub fn i2c_smbus_read_byte(file: c_int) -> c_int;
    pub fn i2c_smbus_write_byte(file: c_int, value: u8) -> c_int;
    pub fn i2c_smbus_read_byte_data(file: c_int, command: u8) -> c_int;
    pub fn i2c_smbus_write_byte_data(file: c_int, command: u8, value: u8) -> c_int;
    pub fn i2c_smbus_read_word_data(file: c_int, command: u8) -> c_int;
    pub fn i2c_smbus_write_word_data(file: c_int, command: u8, value: u16) -> c_int;
    pub fn i2c_smbus_process_call(file: c_int, command: u8, value: u16) -> c_int;
    pub fn i2c_smbus_read_block_data(file: c_int, command: u8, values: *mut u8) -> c_int;
    pub fn i2c_smbus_write_block_data(
        file: c_int,
        command: u8,
        length: u8,
        values: *const u8,
    ) -> c_int;
    pub fn i2c_smbus_read_i2c_block_data(
        file: c_int,
        command: u8,
        length: u8,
        values: *mut u8,
    ) -> c_int;
    pub fn i2c_smbus_write_i2c_block_data(
        file: c_int,
        command: u8,
        length: u8,
        values: *const u8,
    ) -> c_int;
    pub fn i2c_smbus_block_process_call(
        file: c_int,
        command: u8,
        length: u8,
        values: *mut u8,
    ) -> c_int;
}
