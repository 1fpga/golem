# Notes

## Reset

There are 4 reset signals per se.
1. FPGA Reset
2. HPS Reset
3. Core Reset
4. Firmware Reset (Restart)

Cold reboot will trigger the HPS reset signal which in will reprogram the FPGA via uboot during the boot, the hot reboot is just restarting the firmware.

The menu (`menu.rbf`) is just the framework with a noise generator and an id for the firmware to treat it as a menu. It's an empty shell
