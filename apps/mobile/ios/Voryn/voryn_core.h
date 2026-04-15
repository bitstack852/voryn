#ifndef VORYN_CORE_H
#define VORYN_CORE_H

#include <stdint.h>
#include <stddef.h>

const char* voryn_hello(void);
const char* voryn_generate_identity(void);
const char* voryn_compute_safety_number(const uint8_t* our_pk, const uint8_t* their_pk);
void voryn_free_string(const char* s);

#endif
