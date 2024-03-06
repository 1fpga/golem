#include "hardware.h"
#include "offload.h"

extern "C" unsigned long GetTimer(unsigned long offset) {return 0;}
extern "C" unsigned long CheckTimer(unsigned long t) {return 0;}
extern "C" void WaitTimer(unsigned long time) {}

extern "C" void offload_start() {}
extern "C" void offload_stop() {}
extern "C" void offload_add_work(std::function<void()> work) {}
