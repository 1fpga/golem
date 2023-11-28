#include <stdint.h>

#ifndef SMBUS_H
#define SMBUS_H

extern "C" int i2c_open(int dev_address, int is_smbus);
extern "C" void i2c_close(int fd);

extern "C" int i2c_smbus_write_quick(int file, uint8_t value);
extern "C" int i2c_smbus_read_byte(int file);
extern "C" int i2c_smbus_write_byte(int file, uint8_t value);
extern "C" int i2c_smbus_read_byte_data(int file, uint8_t command);
extern "C" int i2c_smbus_write_byte_data(int file, uint8_t command, uint8_t value);
extern "C" int i2c_smbus_read_word_data(int file, uint8_t command);
extern "C" int i2c_smbus_write_word_data(int file, uint8_t command, uint16_t value);
extern "C" int i2c_smbus_process_call(int file, uint8_t command, uint16_t value);
extern "C" int i2c_smbus_read_block_data(int file, uint8_t command, uint8_t *values);
extern "C" int i2c_smbus_write_block_data(int file, uint8_t command, uint8_t length, const uint8_t *values);
extern "C" int i2c_smbus_read_i2c_block_data(int file, uint8_t command, uint8_t length, uint8_t *values);
extern "C" int i2c_smbus_write_i2c_block_data(int file, uint8_t command, uint8_t length, const uint8_t *values);
extern "C" int i2c_smbus_block_process_call(int file, uint8_t command, uint8_t length, uint8_t *values);

#endif
