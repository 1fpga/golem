#ifndef OFFLOAD_H
#define OFFLOAD_H

#include <stddef.h>
#include <functional>

extern "C" void offload_start();
extern "C" void offload_stop();

void offload_add_work(std::function<void()> work);

#endif
