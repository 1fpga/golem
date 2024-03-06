/*
Copyright 2005, 2006, 2007 Dennis van Weeren
Copyright 2008, 2009 Jakub Bednarski

This file is part of Minimig

Minimig is free software; you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation; either version 3 of the License, or
(at your option) any later version.

Minimig is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

// 2009-11-14   - OSD labels changed
// 2009-12-15   - added display of directory name extensions
// 2010-01-09   - support for variable number of tracks
// 2016-06-01   - improvements to 8-bit menu

#include <stdlib.h>
#include <inttypes.h>
#include <ctype.h>
#include <fcntl.h>
#include <time.h>
#include <arpa/inet.h>
#include <sys/socket.h>
#include <netdb.h>
#include <ifaddrs.h>
#include <sys/stat.h>
#include <sys/statvfs.h>
#include <stdbool.h>
#include <stdio.h>
#include <sched.h>
#include <string.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <libgen.h>
#include <bluetooth.h>
#include <hci.h>
#include <hci_lib.h>

#include "file_io.h"
#include "osd.h"
#include "hardware.h"
#include "menu.h"
#include "user_io.h"
#include "debug.h"
#include "fpga_io.h"
#include "cfg.h"
#include "input.h"
#include "battery.h"
#include "cheats.h"
#include "video.h"
#include "audio.h"
#include "joymapping.h"
#include "recent.h"
#include "support.h"
#include "bootcore.h"
#include "ide.h"
#include "profiling.h"


void OsdSetTitle(const char *s, int arrow ) {}
void OsdSetArrow(int arrow) {}
void OsdWrite(unsigned char n, const char *s, unsigned char inver, unsigned char stipple, char usebg , int maxinv , int mininv ) {}
void OsdWriteOffset(unsigned char n, const char *s, unsigned char inver, unsigned char stipple, char offset, char leftchar, char usebg , int maxinv , int mininv ) {}
void OsdClear() {}
void OsdEnable(unsigned char mode) {}
void InfoEnable(int x, int y, int width, int height) {}
void OsdRotation(uint8_t rotate) {}
void OsdDisable() {}
void OsdMenuCtl(int en) {}
void OsdUpdate() {}
void OSD_PrintInfo(const char *message, int *width, int *height, int frame ) {}
void OsdDrawLogo(int row) {}
void ScrollText(char n, const char *str, int off, int len, int max_len, unsigned char invert, int idx ) {}
void ScrollReset(int idx ) {}
void StarsInit() {}
void StarsUpdate() {}
void OsdShiftDown(unsigned char n){}

// get/set core currently loaded
void OsdCoreNameSet(const char* str) {}
const char* OsdCoreNameGet() {return "";}
void OsdSetSize(int n){}
int OsdGetSize() {return 8;}

/*menu states*/
enum MENU
{
	MENU_NONE1,
	MENU_NONE2,
	MENU_INFO,

	MENU_SYSTEM1,
	MENU_SYSTEM2,
	MENU_COMMON1,
	MENU_COMMON2,
	MENU_MISC1,
	MENU_MISC2,

	MENU_FILE_SELECT1,
	MENU_FILE_SELECT2,
	MENU_CORE_FILE_SELECTED1,
	MENU_CORE_FILE_SELECTED2,
	MENU_CORE_FILE_CANCELED,
	MENU_RECENT1,
	MENU_RECENT2,
	MENU_RECENT3,
	MENU_RECENT4,
	MENU_ABOUT1,
	MENU_ABOUT2,
	MENU_RESET1,
	MENU_RESET2,

	MENU_JOYSYSMAP,
	MENU_JOYDIGMAP,
	MENU_JOYDIGMAP1,
	MENU_JOYDIGMAP2,
	MENU_JOYDIGMAP3,
	MENU_JOYDIGMAP4,
	MENU_JOYRESET,
	MENU_JOYRESET1,
	MENU_JOYKBDMAP,
	MENU_JOYKBDMAP1,
	MENU_KBDMAP,
	MENU_KBDMAP1,
	MENU_BTPAIR,
	MENU_BTPAIR2,
	MENU_LGCAL,
	MENU_LGCAL1,
	MENU_LGCAL2,

	MENU_SCRIPTS_PRE,
	MENU_SCRIPTS_PRE1,
	MENU_SCRIPTS,
	MENU_SCRIPTS1,
	MENU_SCRIPTS_FB,
	MENU_SCRIPTS_FB2,

	MENU_DOC_FILE_SELECTED,
	MENU_DOC_FILE_SELECTED_2,

	MENU_CHEATS1,
	MENU_CHEATS2,

	MENU_UART1,
	MENU_UART2,
	MENU_UART3,
	MENU_UART4,
	MENU_BAUD1,
	MENU_BAUD2,

	MENU_SFONT_FILE_SELECTED,

	MENU_VIDEOPROC1,
	MENU_VIDEOPROC2,
	MENU_COEFF_FILE_SELECTED,
	MENU_GAMMA_FILE_SELECTED,
	MENU_SMASK_FILE_SELECTED,
	MENU_PRESET_FILE_SELECTED,

	MENU_AFILTER_FILE_SELECTED,

	// Generic
	MENU_GENERIC_MAIN1,
	MENU_GENERIC_MAIN2,
	MENU_GENERIC_FILE_SELECTED,
	MENU_GENERIC_IMAGE_SELECTED,
	MENU_GENERIC_SAVE_WAIT,

	// Arcade
	MENU_ARCADE_DIP1,
	MENU_ARCADE_DIP2,

	// Minimig
	MENU_MINIMIG_MAIN1,
	MENU_MINIMIG_MAIN2,
	MENU_MINIMIG_VIDEO1,
	MENU_MINIMIG_VIDEO2,
	MENU_MINIMIG_CHIPSET1,
	MENU_MINIMIG_CHIPSET2,
	MENU_MINIMIG_DISK1,
	MENU_MINIMIG_DISK2,
	MENU_MINIMIG_HDFFILE_SELECTED,
	MENU_MINIMIG_ADFFILE_SELECTED,
	MENU_MINIMIG_ROMFILE_SELECTED,
	MENU_MINIMIG_LOADCONFIG1,
	MENU_MINIMIG_LOADCONFIG2,
	MENU_MINIMIG_SAVECONFIG1,
	MENU_MINIMIG_SAVECONFIG2,

	// Atari ST
	MENU_ST_MAIN1,
	MENU_ST_MAIN2,
	MENU_ST_SYSTEM1,
	MENU_ST_SYSTEM2,
	MENU_ST_FDD_FILE_SELECTED,
	MENU_ST_HDD_FILE_SELECTED,
	MENU_ST_SYSTEM_FILE_SELECTED,
	MENU_ST_LOAD_CONFIG1,
	MENU_ST_LOAD_CONFIG2,
	MENU_ST_SAVE_CONFIG1,
	MENU_ST_SAVE_CONFIG2,

	// Archie
	MENU_ARCHIE_MAIN1,
	MENU_ARCHIE_MAIN2,
	MENU_ARCHIE_MAIN_FILE_SELECTED,

	// MT32-pi
	MENU_MT32PI_MAIN1,
	MENU_MT32PI_MAIN2,
};

static uint32_t menustate = MENU_NONE1;
static uint32_t parentstate;
static uint32_t menusub = 0;
static uint32_t menusub_last = 0; //for when we allocate it dynamically and need to know last row
static uint64_t menumask = 0; // Used to determine which rows are selectable...
static uint32_t menu_timer = 0;
static uint32_t menu_save_timer = 0;
static uint32_t load_addr = 0;
static int32_t  bt_timer = 0;

extern const char *version;

const char *config_tos_wrprot[] = { "None", "A:", "B:", "A: and B:" };

const char *config_scanlines_msg[] = { "Off", "HQ2x", "CRT 25%" , "CRT 50%" , "CRT 75%" };
const char *config_blank_msg[] = { "Blank", "Blank+" };
const char *config_dither_msg[] = { "off", "SPT", "RND", "S+R" };
const char *config_autofire_msg[] = { "        AUTOFIRE OFF", "        AUTOFIRE FAST", "        AUTOFIRE MEDIUM", "        AUTOFIRE SLOW" };
const char *config_joystick_mode[] = { "Digital", "Analog", "CD32", "Analog" };
const char *config_button_turbo_msg[] = { "OFF", "FAST", "MEDIUM", "SLOW" };
const char *config_button_turbo_choice_msg[] = { "A only", "B only", "A & B" };
const char *joy_button_map[] = { "RIGHT", "LEFT", "DOWN", "UP", "BUTTON A", "BUTTON B", "BUTTON X", "BUTTON Y", "BUTTON L", "BUTTON R", "SELECT", "START", "KBD TOGGLE", "MENU", "    Stick 1: Tilt RIGHT", "    Stick 1: Tilt DOWN", "   Mouse emu X: Tilt RIGHT", "   Mouse emu Y: Tilt DOWN" };
const char *joy_ana_map[] = { "    DPAD test: Press RIGHT", "    DPAD test: Press DOWN", "   Stick 1 Test: Tilt RIGHT", "   Stick 1 Test: Tilt DOWN", "   Stick 2 Test: Tilt RIGHT", "   Stick 2 Test: Tilt DOWN" };
const char *config_stereo_msg[] = { "0%", "25%", "50%", "100%" };
const char *config_uart_msg[] = { "      None", "       PPP", "   Console", "      MIDI", "     Modem"};
const char *config_midilink_mode[] = {"Local", "Local", "  USB", "  UDP", "-----", "-----", "  USB" };
const char *config_afilter_msg[] = { "Internal","Custom" };
const char *config_smask_msg[] = { "None", "1x", "2x", "1x Rotated", "2x Rotated" };
const char *config_scale[] = { "Normal", "V-Integer", "HV-Integer-", "HV-Integer+", "HV-Integer", "???", "???", "???" };

#define DPAD_NAMES 4
#define DPAD_BUTTON_NAMES 12  //DPAD_NAMES + 6 buttons + start/select

#define script_line_length 1024
#define script_lines 50
static FILE *script_pipe;
static int script_file;
static char script_command[script_line_length];
static int script_line;
static char script_output[script_lines][script_line_length];
static char script_line_output[script_line_length];
static bool script_finished;

// one screen width
static const char* HELPTEXT_SPACER = "                                ";
static char helptext_custom[1024];

enum HelpText_Message
{
	HELPTEXT_NONE, HELPTEXT_CUSTOM, HELPTEXT_MAIN, HELPTEXT_HARDFILE, HELPTEXT_CHIPSET, HELPTEXT_MEMORY, HELPTEXT_EJECT, HELPTEXT_CLEAR
};

static const char *helptexts[] =
{
	0,
	helptext_custom,
	"                                Welcome to MiSTer! Use the cursor keys to navigate the menus. Use space bar or enter to select an item. Press Esc or F12 to exit the menus. Joystick emulation on the numeric keypad can be toggled with the numlock or scrlock key, while pressing Ctrl-Alt-0 (numeric keypad) toggles autofire mode.",
	"                                Minimig can emulate an A600/A1200 IDE harddisk interface. The emulation can make use of Minimig-style hardfiles (complete disk images) or UAE-style hardfiles (filesystem images with no partition table).",
	"                                Minimig's processor core can emulate a 68000 (cycle accuracy as A500/A600) or 68020 (maximum performance) processor with transparent cache.",
	"                                Minimig can make use of up to 2 megabytes of Chip RAM, up to 1.5 megabytes of Slow RAM (A500 Trapdoor RAM), and up to 384 megabytes of Fast RAM (8MB max for 68000 mode). To use the HRTmon feature you will need a file on the SD card named hrtmon.rom.",
	"                                Backspace key (or B-hold + A on gamepad) to unmount",
	"                                Backspace key (or B-hold + A on gamepad) to clear stored option. You have to reload the core to be able to use default value.",
};

static const uint32_t helptext_timeouts[] =
{
	10000,
	10000,
	10000,
	10000,
	10000,
	10000,
	10000,
	10000
};

static const char *info_top = "\x80\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x82";
static const char *info_bottom = "\x85\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x81\x84";

// file selection menu variables
static char fs_pFileExt[13] = "xxx";
static uint32_t fs_ExtLen = 0;
static uint32_t fs_Options;
static uint32_t fs_MenuSelect;
static uint32_t fs_MenuCancel;

static char* GetExt(char *ext)
{
	static char extlist[32];
	char *p = extlist;

	while (*ext) {
		strcpy(p, ",");
		strncat(p, ext, 3);
		while (*(p + strlen(p) - 1) == ' ') *(p + strlen(p) - 1) = 0;
		if (strlen(ext) <= 3) break;
		ext += 3;
		p += strlen(p);
	}

	return extlist + 1;
}

static char SelectedDir[1024] = {};
static char SelectedLabel[1024] = {};

static char Selected_F[16][1024] = {};
static char Selected_S[16][1024] = {};
static char Selected_tmp[1024] = {};

void StoreIdx_F(int idx, const char *path)
{
	strcpy(Selected_F[idx], path);
}

void StoreIdx_S(int idx, const char *path)
{
	strcpy(Selected_S[idx], path);
}

static char selPath[1024] = {};

static int changeDir(char *dir)
{
	char curdir[128];
	memset(curdir, 0, sizeof(curdir));
	if(!dir || !strcmp(dir, ".."))
	{
		if (!strlen(selPath))
		{
			return 0;
		}

		char *p = strrchr(selPath, '/');
		if (p)
		{
			*p = 0;
			strncpy(curdir, p + 1, sizeof(curdir) - 1);
		}
		else
		{
			strncpy(curdir, selPath, sizeof(curdir) - 1);
			selPath[0] = 0;
		}
	}
	else
	{
		if (strlen(selPath) + strlen(dir) > sizeof(selPath) - 100)
		{
			return 0;
		}

		if (strlen(selPath)) strcat(selPath, "/");
		strcat(selPath, dir);
	}

	ScanDirectory(selPath, SCANF_INIT, fs_pFileExt, fs_Options);
	if(curdir[0])
	{
		ScanDirectory(selPath, SCANF_SET_ITEM, curdir, fs_Options);
	}
	return 1;
}

static const char *home_dir = NULL;
static char filter[256] = {};
static unsigned long filter_typing_timer = 0;

// this function displays file selection menu
void SelectFile(const char* path, const char* pFileExt, int Options, unsigned char MenuSelect, unsigned char MenuCancel)
{
	static char tmp[1024];
	printf("pFileExt = %s\n", pFileExt);
	filter_typing_timer = 0;
	filter[0] = 0;

	strncpy(selPath, path, sizeof(selPath) - 1);
	selPath[sizeof(selPath) - 1] = 0;

	if (Options & SCANO_CORES)
	{
		strcpy(selPath, get_rbf_dir());
		if (strlen(get_rbf_name()))
		{
			if(strlen(selPath)) strcat(selPath, "/");
			strcat(selPath, get_rbf_name());
		}
		pFileExt = "RBFMRAMGL";
		home_dir = NULL;
	}
	else if (Options & SCANO_TXT)
	{
		if(pFileExt == 0) pFileExt = "TXT";
		home_dir = NULL;
	}
	else
	{
		const char *home = is_menu() ? "Scripts" : user_io_get_core_path((is_pce() && !strncasecmp(pFileExt, "CUE", 3)) ? PCECD_DIR : NULL, 1);
		home_dir = strrchr(home, '/');
		if (home_dir) home_dir++;
		else home_dir = home;

		if (Options & SCANO_SAVES)
		{
			snprintf(tmp, sizeof(tmp), "%s/%s", SAVE_DIR, CoreName);
			home = tmp;
		}

		if (strncasecmp(home, selPath, strlen(home)) || !strcasecmp(home, selPath) || (!FileExists(selPath) && !PathIsDir(selPath)))
		{
			Options &= ~SCANO_NOENTER;
			strcpy(selPath, home);
		}
	}

	ScanDirectory(selPath, SCANF_INIT, pFileExt, Options);
	AdjustDirectory(selPath);

	strcpy(fs_pFileExt, pFileExt);
	fs_ExtLen = strlen(fs_pFileExt);
	fs_Options = Options & ~SCANO_NOENTER;
	fs_MenuSelect = MenuSelect;
	fs_MenuCancel = MenuCancel;

	menustate = MENU_FILE_SELECT1;
}

#define STD_EXIT       "            exit"
#define STD_BACK       "            back"
#define STD_SPACE_EXIT "        SPACE to exit"
#define STD_COMBO_EXIT "      Ctrl+ESC to exit"

// conversion table of Amiga keyboard scan codes to ASCII codes
static const uint8_t keycode_table[128] =
{
	0,'1','2','3','4','5','6','7','8','9','0',  0,  0,  0,  0,  0,
	'Q','W','E','R','T','Y','U','I','O','P',  0,  0,  0,  0,  0,  0,
	'A','S','D','F','G','H','J','K','L',  0,  0,  0,  0,  0,  0,  0,
	0,'Z','X','C','V','B','N','M',  0,  0,  0,  0,  0,  0,  0,  0,
	0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  1,  1,  1,  1,
	0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
	0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  1,  1,  0,  0,
	0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0
};

static uint8_t GetASCIIKey(uint32_t keycode)
{
	if (keycode & UPSTROKE)
		return 0;

	return keycode_table[get_amiga_code(keycode & 0xFFFF) & 0x7F];
}

/* the Atari core handles OSD keys competely inside the core */
static uint32_t menu_key = 0;

void menu_key_set(unsigned int c)
{
	//printf("OSD enqueue: %x\n", c);
	menu_key = c;
}

// get key status
static int hold_cnt = 0;
static uint32_t menu_key_get(void)
{
	static uint32_t prev_key = 0;
	static unsigned long db_time = 0;
	if (prev_key != menu_key || !db_time)
	{
		prev_key = menu_key;
		db_time = GetTimer(20);
	}

	uint32_t c = 0;
	if (CheckTimer(db_time))
	{
		static uint32_t c2;
		static unsigned long repeat;
		uint32_t c1;

		c1 = menu_key;
		c = 0;
		if (c1 != c2)
		{
			c = c1;
			hold_cnt = 1;
		}
		c2 = c1;

		// generate repeat "key-pressed" events
		if ((c1 & UPSTROKE) || (!c1))
		{
			hold_cnt = 0;
			repeat = GetTimer(REPEATDELAY);
		}
		else if (CheckTimer(repeat))
		{
			repeat = GetTimer(REPEATRATE);
			if (GetASCIIKey(c1) || ((menustate == MENU_COMMON2) && (menusub == 17)) || ((menustate == MENU_SYSTEM2) && (menusub == 5)))
			{
				c = c1;
				hold_cnt++;
			}
		}
	}

	// currently no key pressed
	if (!c)
	{
		static unsigned long longpress = 0, longpress_consumed = 0;
		static unsigned char last_but = 0;
		unsigned char but = user_io_menu_button();

		if (but && !last_but) longpress = GetTimer(3000);
		if (but && CheckTimer(longpress) && !longpress_consumed)
		{
			longpress_consumed = 1;
			if (menustate == MENU_SCRIPTS1) c = KEY_BACKSPACE;
			else menustate = MENU_BTPAIR;
		}

		if (!but && last_but && !longpress_consumed) c = KEY_F12;

		if (!but) longpress_consumed = 0;
		last_but = but;
	}

	if (!c)
	{
		static unsigned long longpress = 0, longpress_consumed = 0;
		static unsigned char last_but = 0;
		unsigned char but = user_io_user_button();

		if (user_io_osd_is_visible())
		{
			if (but && !last_but) longpress = GetTimer(1500);
			if (but && CheckTimer(longpress) && !longpress_consumed)
			{
				longpress_consumed = 1;
				if (is_menu())
				{
					if (menustate == MENU_SYSTEM2 || menustate == MENU_FILE_SELECT2) menustate = MENU_JOYSYSMAP;
				}
				else if (get_map_vid() || get_map_pid())
				{
					menustate = MENU_JOYRESET;
				}
			}

			if (!but && last_but && !longpress_consumed)
			{
				if (get_map_vid() || get_map_pid())
				{
					send_map_cmd(KEY_ALTERASE);
				}
			}
		}

		if (!but) longpress_consumed = 0;
		last_but = but;
	}

	return(c);
}

static char* getNet(int spec)
{
	int netType = 0;
	struct ifaddrs *ifaddr, *ifa, *ifae = 0, *ifaw = 0;
	static char host[NI_MAXHOST];

	if (getifaddrs(&ifaddr) == -1)
	{
		printf("getifaddrs: error\n");
		return NULL;
	}

	for (ifa = ifaddr; ifa != NULL; ifa = ifa->ifa_next)
	{
		if (ifa->ifa_addr == NULL) continue;
		if (!memcmp(ifa->ifa_addr->sa_data, "\x00\x00\xa9\xfe", 4)) continue; // 169.254.x.x

		if ((strcmp(ifa->ifa_name, "eth0") == 0)     && (ifa->ifa_addr->sa_family == AF_INET)) ifae = ifa;
		if ((strncmp(ifa->ifa_name, "wlan", 4) == 0) && (ifa->ifa_addr->sa_family == AF_INET)) ifaw = ifa;
	}

	ifa = 0;
	netType = 0;
	if (ifae && (!spec || spec == 1))
	{
		ifa = ifae;
		netType = 1;
	}

	if (ifaw && (!spec || spec == 2))
	{
		ifa = ifaw;
		netType = 2;
	}

	if (spec && ifa)
	{
		strcpy(host, "IP: ");
		getnameinfo(ifa->ifa_addr, sizeof(struct sockaddr_in), host + strlen(host), NI_MAXHOST - strlen(host), NULL, 0, NI_NUMERICHOST);
	}

	freeifaddrs(ifaddr);
	return spec ? (ifa ? host : 0) : (char*)netType;
}

static long sysinfo_timer;
static void infowrite(int pos, const char* txt)
{
	char str[40];
	memset(str, 0x20, 29);
	int len = strlen(txt);
	if (len > 27) len = 27;
	if(len) strncpy(str + 1+ ((27-len)/2), txt, len);
	str[0] = 0x83;
	str[28] = 0x83;
	str[29] = 0;
	OsdWrite(pos, str, 0, 0);
}

static void printSysInfo()
{
	if (!sysinfo_timer || CheckTimer(sysinfo_timer))
	{
		sysinfo_timer = GetTimer(2000);
		struct battery_data_t bat;
		int hasbat = getBattery(0, &bat);
		int n = 2;
		static int flip = 0;

		char str[40];
		OsdWrite(n++, info_top, 0, 0);

		int j = 0;
		char *net;
		net = getNet(1);
		if (net)
		{
			sprintf(str, "\x1c %s", net);
			infowrite(n++, str);
			j++;
		}
		net = getNet(2);
		if (net)
		{
			sprintf(str, "\x1d %s", net);
			infowrite(n++, str);
			j++;
		}
		if (!j) infowrite(n++, "No network");
		if (j<2) infowrite(n++, "");

		flip = (flip + 1) & 3;
		if (hasbat && (flip & 2))
		{
			infowrite(n++, "");

			sprintf(str, "\x1F ");
			if (bat.capacity == -1) strcat(str, "n/a");
			else sprintf(str + strlen(str), "%d%%", bat.capacity);
			if (bat.current != -1) sprintf(str + strlen(str), " %dmAh", bat.current);
			if (bat.voltage != -1) sprintf(str + strlen(str), " %d.%dV", bat.voltage / 1000, (bat.voltage / 100) % 10);

			infowrite(n++, str);

			str[0] = 0;
			if (bat.load_current > 0)
			{
				sprintf(str + strlen(str), " \x12 %dmA", bat.load_current);
				if (bat.time != -1)
				{
					if (bat.time < 90) sprintf(str + strlen(str), ", ETA: %dm", bat.time);
					else sprintf(str + strlen(str), ", ETA: %dh%02dm", bat.time / 60, bat.time % 60);
				}
			}
			else if (bat.load_current < -1)
			{
				sprintf(str + strlen(str), " \x13 %dmA", -bat.load_current);
				if (bat.time != -1)
				{
					if (bat.time < 90) sprintf(str + strlen(str), ", ETA: %dm", bat.time);
					else sprintf(str + strlen(str), ", ETA: %dh%02dm", bat.time / 60, bat.time % 60);
				}
			}
			else
			{
				strcat(str, "Not charging");
			}
			infowrite(n++, str);
		}
		else
		{
			infowrite(n++, "");
			video_core_description(str, 40);
			infowrite(n++, str);
			video_scaler_description(str, 40);
			infowrite(n++, str);
		}
		OsdWrite(n++, info_bottom, 0, 0);
	}
}

static int  firstmenu = 0;
static int  adjvisible;

static void MenuWrite(unsigned char n, const char *s = "", unsigned char invert = 0, unsigned char stipple = 0, int arrow = 0)
{
	int row = n - firstmenu;

	if (row < 0)
	{
		if (invert) adjvisible = row;
		return;
	}

	if (row >= OsdGetSize())
	{
		if (invert) adjvisible = row - OsdGetSize() + 1;
		return;
	}

	OsdSetArrow(arrow);
	OsdWriteOffset(row, s, invert, stipple, 0, (row == 0 && firstmenu) ? 17 : (row == (OsdGetSize()-1) && !arrow) ? 16 : 0, 0);
}

const char* get_rbf_name_bootcore(char *str)
{
	if (!strlen(cfg.bootcore)) return "";
	char *p = strrchr(str, '/');
	if (!p) return str;

	char *spl = strrchr(p + 1, '.');
	if (spl && (!strcmp(spl, ".rbf") || !strcmp(spl, ".mra") || !strcmp(spl, ".mgl")))
	{
		*spl = 0;
	}
	else
	{
		return NULL;
	}
	return p + 1;
}

static void vga_nag()
{
}

void process_addon(char *ext, uint8_t idx)
{
	static char name[1024];

	while (*ext && *ext != ',') ext++;
	if (*ext) ext++;
	if (!*ext) return;

	printf("addons: %s\n", ext);

	int i = 0;
	while (1)
	{
		char *fname = name;
		strcpy(name, selPath);
		char *p = strrchr(name, '.');
		if (!p) p = name + strlen(name);
		*p++ = '.';

		substrcpy(p, ext, i);
		if (!strlen(p)) return;
		if (*p == '!')
		{
			*p = 0;
			char *bs = strrchr(name, '/');
			if (!bs)
			{
				fname = p + 1;
			}
			else
			{
				strcpy(bs + 1, p + 1);
			}
		}

		printf("Trying: %s\n", fname);
		user_io_file_tx_a(fname, ((i+1) << 8) | idx);
		i++;
	}
}

static int get_arc(const char *str)
{
	int arc = 0;
	if (!strcmp(str, "[ARC1]")) arc = 1;
	else if(!strcmp(str, "[ARC2]")) arc = 2;
	else return 0;

	uint32_t x = 0, y = 0;
	if (sscanf(cfg.custom_aspect_ratio[arc - 1], "%u:%u", &x, &y) != 2 || x < 1 || x > 4095 || y < 1 || y > 4095) arc = -1;

	return arc;
}

static int get_ar_name(int ar, char *str)
{
	switch (ar)
	{
	case 0:
		strcat(str, "Original");
		break;

	case 1:
		strcat(str, "Full Screen");
		break;

	case 2:
		if (get_arc("[ARC1]") <= 0)
		{
			strcat(str, "Original");
			ar = 0;
		}
		else
		{
			strcat(str, cfg.custom_aspect_ratio[0]);
		}
		break;

	case 3:
		if (get_arc("[ARC2]") <= 0)
		{
			strcat(str, "Original");
			ar = 0;
		}
		else
		{
			strcat(str, cfg.custom_aspect_ratio[1]);
		}
		break;
	}

	return ar;
}

static int next_ar(int ar, int minus)
{
	if (minus)
	{
		ar = (ar - 1) & 3;
		while (1)
		{
			if (ar == 3 && get_arc("[ARC2]") > 0 && get_arc("[ARC1]") > 0) break;
			if (ar == 2 && get_arc("[ARC1]") > 0) break;
			if (ar < 2) break;
			ar--;
		}
	}
	else
	{
		ar = (ar + 1) & 3;
		if (ar == 3 && get_arc("[ARC2]") <= 0) ar = 0;
		if (ar == 2 && get_arc("[ARC1]") <= 0) ar = 0;
	}

	return ar;
}

static int joymap_first = 0;

static int gun_x = 0;
static int gun_y = 0;
static int gun_ok = 0;
static int gun_side = 0;
static int gun_idx = 0;
static int32_t gun_pos[4] = {};
static int page = 0;

void HandleUI(void)
{}

void open_joystick_setup()
{

}

void ScrollLongName(void)
{
	// this function is called periodically when file selection window is displayed
	// it checks if predefined period of time has elapsed and scrolls the name if necessary

	int off = 0;
	int max_len;

	int len = strlen(flist_SelectedItem()->altname); // get name length

	max_len = 30; // number of file name characters to display (one more required for scrolling)
	if (flist_SelectedItem()->de.d_type == DT_DIR)
	{
		max_len = 23; // number of directory name characters to display
		if ((fs_Options & SCANO_CORES) && (flist_SelectedItem()->altname[0] == '_'))
		{
			off = 1;
			len--;
		}
	}

	if (flist_SelectedItem()->de.d_type != DT_DIR) // if a file
	{
		if (!cfg.rbf_hide_datecode && flist_SelectedItem()->datecode[0])
		{
			max_len = 20; // __.__.__ remove that from the end
		}
		else if (cfg.browse_expand && len < 55)
		{
			return;
		}
	}

	ScrollText(flist_iSelectedEntry() - flist_iFirstEntry(), flist_SelectedItem()->altname + off, 0, len, max_len, 1);
}

// print directory contents
void PrintDirectory(int expand)
{
	char s[40];
	ScrollReset();

	if (!cfg.browse_expand) expand = 0;

	if (expand)
	{
		int k = flist_iFirstEntry() + OsdGetSize() - 1;
		if (flist_nDirEntries() && k == flist_iSelectedEntry() && k <= flist_nDirEntries()
			&& strlen(flist_DirItem(k)->altname) > 28 && !(!cfg.rbf_hide_datecode && flist_DirItem(k)->datecode[0])
			&& flist_DirItem(k)->de.d_type != DT_DIR)
		{
			//make room for last expanded line
			flist_iFirstEntryInc();
		}
	}

	int i = 0;
	int k = flist_iFirstEntry();
	while(i < OsdGetSize())
	{
		char leftchar = 0;
		memset(s, ' ', 32); // clear line buffer
		s[32] = 0;
		int len2 = 0;
		leftchar = 0;
		int len = 0;

		if (k < flist_nDirEntries())
		{
			len = strlen(flist_DirItem(k)->altname); // get name length
			if (len > 28)
			{
				len2 = len - 27;
				if (len2 > 27) len2 = 27;
				if (!expand) len2 = 0;

				len = 27; // trim display length if longer than 30 characters
				s[28] = 22;
			}

			if((flist_DirItem(k)->de.d_type == DT_DIR) && (fs_Options & SCANO_CORES) && (flist_DirItem(k)->altname[0] == '_'))
			{
				strncpy(s + 1, flist_DirItem(k)->altname+1, len-1);
			}
			else
			{
				strncpy(s + 1, flist_DirItem(k)->altname, len); // display only name
			}

			char *datecode = flist_DirItem(k)->datecode;
			if (flist_DirItem(k)->de.d_type == DT_DIR) // mark directory with suffix
			{
				if (!strcmp(flist_DirItem(k)->altname, ".."))
				{
					strcpy(&s[19], " <UP-DIR>");
				}
				else
				{
					strcpy(&s[22], " <DIR>");
				}
				len2 = 0;
			}
			else if (!cfg.rbf_hide_datecode && datecode[0])
			{
				int n = 19;
				s[n++] = ' ';
				s[n++] = datecode[0];
				s[n++] = datecode[1];
				s[n++] = '.';
				s[n++] = datecode[2];
				s[n++] = datecode[3];
				s[n++] = '.';
				s[n++] = datecode[4];
				s[n++] = datecode[5];

				if (len >= 19)
				{
					s[19] = 22;
					s[28] = ' ';
				}
				len2 = 0;
			}

			if (!i && k) leftchar = 17;
			if (i && k < flist_nDirEntries() - 1) leftchar = 16;
		}
		else if(!flist_nDirEntries()) // selected directory is empty
		{
			if (!i) strcpy(s, "          No files!");
			if (home_dir && !filter[0])
			{
				if (i == 6) strcpy(s, "      Missing directory:");
				if (i == 8)
				{
					len = strlen(home_dir);
					if (len > 27) len = 27;
					strncpy(s + 1 + ((27 - len) / 2), home_dir, len);
				}
			}
		}

		int sel = (i == (flist_iSelectedEntry() - flist_iFirstEntry()));
		OsdWriteOffset(i, s, sel, 0, 0, leftchar);
		i++;

		if (sel && len2)
		{
			len = strlen(flist_DirItem(k)->altname);
			strcpy(s+1, flist_DirItem(k)->altname + len - len2);
			OsdWriteOffset(i, s, sel, 0, 0, leftchar);
			i++;
		}

		k++;
	}
}

static void set_text(const char *message, unsigned char code)
{
	char s[40];
	int i = 0, l = 1;

	OsdWrite(0, "", 0, 0);

	do
	{
		s[i++] = *message;

		// line full or line break
		if ((i == 29) || (*message == '\n') || !*message)
		{
			s[--i] = 0;
			OsdWrite(l++, s, 0, 0);
			i = 0;  // start next line
		}
	} while (*message++);

	if (code && (l <= 7))
	{
		sprintf(s, " Code: #%d", code);
		OsdWrite(l++, s, 0, 0);
	}

	while (l <= 7) OsdWrite(l++, "", 0, 0);
}

void InfoMessage(const char *message, int timeout, const char *title)
{
}

void MenuHide()
{
	menustate = MENU_NONE1;
	HandleUI();
}

void Info(const char *message, int timeout, int width, int height, int frame)
{
	if (menustate <= MENU_INFO)
	{
		OSD_PrintInfo(message, &width, &height, frame);
		InfoEnable(20, (cfg.direct_video && get_vga_fb()) ? 30 : 10, width, height);
		OsdSetSize(16);

		menu_timer = GetTimer(timeout);
		menustate = MENU_INFO;
		OsdUpdate();
	}
}

int menu_lightgun_cb(int idx, uint16_t type, uint16_t code, int value)
{
	if (type == EV_ABS)
	{
		if (code == 0 && value) gun_x = value;
		if (code == 1 && value != 1023) gun_y = value;
	}

	if (type == EV_KEY)
	{
		if ((code == 0x130 || code == 0x131 || code == 0x120) && menustate == MENU_LGCAL1)
		{
			gun_idx = idx;
			if (value == 1) gun_ok = 1;
			if (value == 0) gun_ok = 2;
			return 1;
		}
	}
	return 0;
}

int menu_allow_cfg_switch()
{
	if (user_io_osd_is_visible())
	{
		switch (menustate)
		{
		case MENU_ST_MAIN2:
		case MENU_ARCHIE_MAIN2:
		case MENU_MINIMIG_MAIN2:
		case MENU_COMMON2:
		case MENU_SYSTEM2:
			return 1;

		case MENU_FILE_SELECT2:
			if (is_menu() && (fs_Options & SCANO_CORES)) return 1;
			break;

		case MENU_GENERIC_MAIN2:
			if (!page) return 1;
			break;
		}
	}

	return 0;
}

void menu_process_save()
{
	menu_save_timer = GetTimer(1000);
}

static char pchar[] = { 0x8C, 0x8E, 0x8F, 0x90, 0x91, 0x7F };

#define PROGRESS_CNT    28
#define PROGRESS_CHARS  (int)(sizeof(pchar)/sizeof(pchar[0]))
#define PROGRESS_MAX    ((PROGRESS_CHARS*PROGRESS_CNT)-1)

void ProgressMessage(const char* title, const char* text, int current, int max)
{
	static int progress;
	if (!current && !max)
	{
		progress = -1;
		MenuHide();
		return;
	}

	int new_progress = (((uint64_t)current)*PROGRESS_MAX) / max;
	if (progress != new_progress)
	{
		progress = new_progress;
		static char progress_buf[128];
		memset(progress_buf, 0, sizeof(progress_buf));

		if (new_progress > PROGRESS_MAX) new_progress = PROGRESS_MAX;
		char c = pchar[new_progress % PROGRESS_CHARS];
		new_progress /= PROGRESS_CHARS;

		char *buf = progress_buf;
		sprintf(buf, "\n\n %.27s\n ", text);
		buf += strlen(buf);

		for (int i = 0; i <= new_progress; i++) buf[i] = (i < new_progress) ? 0x7F : c;
		buf[PROGRESS_CNT] = 0;

		InfoMessage(progress_buf, 2000, title);
	}
}
