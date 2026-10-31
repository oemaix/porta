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
// -1 when unchanged, 0 when the session is playing, 1 when it is paused.
int porta_connect_take_pause(void);
// 1 once after a flush. The queued samples are already dropped.
int porta_connect_take_flush(void);

#ifdef __cplusplus
}
#endif
