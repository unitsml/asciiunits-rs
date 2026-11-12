#ifndef ASCIIUNITS_H
#define ASCIIUNITS_H
#include <stddef.h>
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
uint32_t asciiunits_abi_version(void);
/* mode: 0 = raw shape, 1 = structural model. Input bytes are borrowed.
 * The returned NUL-terminated UTF-8 JSON belongs to the caller. */
char *asciiunits_parse_json_v1(const uint8_t *input, size_t length, uint32_t mode);
void asciiunits_free_json_v1(char *json);
#ifdef __cplusplus
}
#endif
#endif
