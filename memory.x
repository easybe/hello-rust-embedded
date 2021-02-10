MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  /* TODO Adjust these memory regions to match your device memory layout */
  FLASH : ORIGIN = 0x08000000, LENGTH = 256K
  RAM : ORIGIN = 0x20000000, LENGTH = 32K
}

_stack_size = 0x2000;

/* `.` is right after the .bss and .data sections */
_heap_start = .;
_heap_end = ORIGIN(RAM) + LENGTH(RAM) - _stack_size;
