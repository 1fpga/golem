#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include <errno.h>
#include <signal.h>
#include <ctype.h>
#include <termios.h>
#include <sys/types.h>
#include <sys/stat.h>

#include "fpga_io.h"
#include "file_io.h"
#include "input.h"
#include "osd.h"
#include "menu.h"
#include "shmem.h"
#include "offload.h"

#include "fpga_base_addr_ac5.h"
#include "fpga_manager.h"
#include "fpga_system_manager.h"
#include "fpga_reset_manager.h"
#include "fpga_nic301.h"

#define FPGA_REG_BASE 0xFF000000
#define FPGA_REG_SIZE 0x01000000

#define MAP_ADDR(x) (volatile uint32_t*)(&map_base[(((uint32_t)(x)) & 0xFFFFFF)>>2])
#define IS_REG(x) (((((uint32_t)(x))-1)>=(FPGA_REG_BASE - 1)) && ((((uint32_t)(x))-1)<(FPGA_REG_BASE + FPGA_REG_SIZE - 1)))

static struct socfpga_reset_manager  *reset_regs   = (socfpga_reset_manager *)SOCFPGA_RSTMGR_ADDRESS;
static struct socfpga_fpga_manager   *fpgamgr_regs = (socfpga_fpga_manager *)SOCFPGA_FPGAMGRREGS_ADDRESS;
static struct socfpga_system_manager *sysmgr_regs  = (socfpga_system_manager *)SOCFPGA_SYSMGR_ADDRESS;
static struct nic301_registers       *nic301_regs  = (nic301_registers *)SOCFPGA_L3REGS_ADDRESS;

extern "C" uint32_t *map_base;
uint32_t *map_base = NULL;

#define writel(val, reg) *MAP_ADDR(reg) = val
#define readl(reg) *MAP_ADDR(reg)

#define clrsetbits_le32(addr, clear, set) writel((readl(addr) & ~(clear)) | (set), addr)
#define setbits_le32(addr, set)           writel( readl(addr) | (set), addr)
#define clrbits_le32(addr, clear)         writel( readl(addr) & ~(clear), addr)

/* Timeout count */
#define FPGA_TIMEOUT_CNT		0x1000000

/* Set CD ratio */
static void fpgamgr_set_cd_ratio(unsigned long ratio)
{
	clrsetbits_le32(&fpgamgr_regs->ctrl,
		0x3 << FPGAMGRREGS_CTRL_CDRATIO_LSB,
		(ratio & 0x3) << FPGAMGRREGS_CTRL_CDRATIO_LSB);
}

extern "C" int fpgamgr_dclkcnt_set_rust(unsigned long);


int fpga_load_rbf(const char *name, const char *cfg, const char *xml)
{
    return 0;
}

static uint32_t gpo_copy = 0;
void inline fpga_gpo_write(uint32_t value)
{
}

void fpga_core_write(uint32_t offset, uint32_t value)
{
}

uint32_t fpga_core_read(uint32_t offset)
{
	return 0;
}

void fpga_set_led(uint32_t on)
{
}

int fpga_get_buttons()
{
}

int fpga_get_io_type()
{
	return 0;
}

void reboot(int cold)
{
}

char *getappname()
{
fprintf(stderr, "!!!!! getappname()\n");
	static char dest[PATH_MAX];
	memset(dest, 0, sizeof(dest));

	char path[64];
	sprintf(path, "/proc/%d/exe", getpid());
	readlink(path, dest, PATH_MAX);

	return dest;
}

void app_restart(const char *path, const char *xml)
{
    abort();
}

void fpga_core_reset(int reset)
{
}

int is_fpga_ready(int quick)
{
	return 1;
}

#define SSPI_STROBE  (1<<17)
#define SSPI_ACK     SSPI_STROBE

uint16_t fpga_spi_fast(uint16_t word)
{
	return 0;
}

void fpga_spi_fast_block_read(uint16_t *buf, uint32_t length)
{
}

void fpga_spi_fast_block_write_8(const uint8_t *buf, uint32_t length)
{
}

void fpga_spi_fast_block_read_8(uint8_t *buf, uint32_t length)
{
}

void fpga_spi_fast_block_write_be(const uint16_t *buf, uint32_t length)
{
}

void fpga_spi_fast_block_read_be(uint16_t *buf, uint32_t length)
{
}
