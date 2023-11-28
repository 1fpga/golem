#ifndef SCHEDULER_H
#define SCHEDULER_H

#define USE_SCHEDULER

extern "C" void scheduler_init(void);
extern "C" void scheduler_run(void);
extern "C" void scheduler_yield(void);

#endif
