#ifndef SHMEM_H
#define SHMEM_H

extern "C" void *shmem_map(uint32_t address, uint32_t size);
extern "C" int shmem_unmap(void* map, uint32_t size);
extern "C" int shmem_put(uint32_t address, uint32_t size, void *buf);
extern "C" int shmem_get(uint32_t address, uint32_t size, void *buf);

#define fpga_mem(x) (0x20000000 | ((x) & 0x1FFFFFFF))
#endif
