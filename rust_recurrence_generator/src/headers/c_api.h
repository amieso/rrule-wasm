#ifndef RUST_RECURRENCE_GENERATOR_H
#define RUST_RECURRENCE_GENERATOR_H

#include <stdio.h>
#include <stdlib.h>

typedef struct {
    char** strings;
    size_t len;
    char* error;
} StringArray;

StringArray* recurrence_generator_generate(const char *rule, const char *after, const char *before);
void free_string_array(StringArray* array);

#endif
