#ifndef tetrad_core_h
#define tetrad_core_h

#include <stdint.h>

#if defined(_WIN32)
  #define EXPORT __declspec(dllexport)
#else
  #define EXPORT
#endif

/* Returns a borrowed, immutable string. Do not free it. */
EXPORT const char* tetrad_init(void);
