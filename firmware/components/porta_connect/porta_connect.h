#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

void porta_connect_start(const char *name, const char *hostname, const char *client_id,
                         const char *client_secret);
int porta_connect_logged_in(void);
void porta_connect_command(int kind, int volume);
size_t porta_connect_take_pcm(int16_t *dst, size_t frames);
size_t porta_connect_take_title(char *dst, size_t cap);

#ifdef __cplusplus
}
#endif
