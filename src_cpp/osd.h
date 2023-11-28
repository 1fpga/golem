#ifndef OSD_H_INCLUDED
#define OSD_H_INCLUDED

#include <inttypes.h>

// some constants
#define DISABLE_KEYBOARD 0x02        // disable keyboard while OSD is active
#define OSD_INFO         0x04        // display info
#define OSD_MSG          0x08        // display message window

#define REPEATDELAY      500         // repeat delay in 1ms units
#define REPEATRATE       50          // repeat rate in 1ms units

#define OSD_ARROW_LEFT   1
#define OSD_ARROW_RIGHT  2

/*functions*/
extern "C" void OsdSetTitle(const char *s, int arrow = 0);	// arrow > 0 = display right arrow in bottom right, < 0 = display left arrow
extern "C" void OsdSetArrow(int arrow);
extern "C" void OsdWrite(unsigned char n, const char *s="", unsigned char inver=0, unsigned char stipple=0, char usebg = 0, int maxinv = 32, int mininv = 0);
extern "C" void OsdWriteOffset(unsigned char n, const char *s, unsigned char inver, unsigned char stipple, char offset, char leftchar, char usebg = 0, int maxinv = 32, int mininv = 0); // Used for scrolling "Exit" text downwards...
extern "C" void OsdClear();
extern "C" void OsdEnable(unsigned char mode);
extern "C" void InfoEnable(int x, int y, int width, int height);
extern "C" void OsdRotation(uint8_t rotate);
extern "C" void OsdDisable();
extern "C" void OsdMenuCtl(int en);
extern "C" void OsdUpdate();
extern "C" void OSD_PrintInfo(const char *message, int *width, int *height, int frame = 0);
extern "C" void OsdDrawLogo(int row);
extern "C" void ScrollText(char n, const char *str, int off, int len, int max_len, unsigned char invert, int idx = 0);
extern "C" void ScrollReset(int idx = 0);
extern "C" void StarsInit();
extern "C" void StarsUpdate();
extern "C" void OsdShiftDown(unsigned char n);

// get/set core currently loaded
extern "C" void OsdCoreNameSet(const char* str);
extern "C" char* OsdCoreNameGet();
extern "C" void OsdSetSize(int n);
extern "C" int OsdGetSize();

#endif

