/* The PSRAM libstdc++ references wide-character helpers that this
   newlib port does not provide. cspot does not call them. */

#include <stdio.h>
#include <string.h>
#include <time.h>
#include <wchar.h>
#include <wctype.h>

#undef getc
#undef getwc
#undef putwc
#undef ungetwc

void porta_wchar_keep(void) {}

wctype_t wctype(const char *property) {
    (void)property;
    return 0;
}

int iswctype(wint_t wc, wctype_t desc) {
    (void)wc;
    (void)desc;
    return 0;
}

wint_t towupper(wint_t wc) { return wc; }

wint_t towlower(wint_t wc) { return wc; }

int wctob(wint_t wc) { return (wc < 128) ? (int)wc : EOF; }

wint_t btowc(int c) { return (c >= 0 && c < 128) ? (wint_t)c : WEOF; }

size_t wcslen(const wchar_t *s) {
    size_t n = 0;
    while (s[n]) {
        n++;
    }
    return n;
}

wchar_t *wmemchr(const wchar_t *s, wchar_t c, size_t n) {
    for (size_t i = 0; i < n; i++) {
        if (s[i] == c) {
            return (wchar_t *)(s + i);
        }
    }
    return NULL;
}

wchar_t *wmemcpy(wchar_t *dst, const wchar_t *src, size_t n) {
    for (size_t i = 0; i < n; i++) {
        dst[i] = src[i];
    }
    return dst;
}

wchar_t *wmemmove(wchar_t *dst, const wchar_t *src, size_t n) {
    if (dst < src) {
        for (size_t i = 0; i < n; i++) {
            dst[i] = src[i];
        }
    } else {
        for (size_t i = n; i > 0; i--) {
            dst[i - 1] = src[i - 1];
        }
    }
    return dst;
}

wchar_t *wmemset(wchar_t *dst, wchar_t c, size_t n) {
    for (size_t i = 0; i < n; i++) {
        dst[i] = c;
    }
    return dst;
}

int strcoll(const char *a, const char *b) { return strcmp(a, b); }

size_t strxfrm(char *dst, const char *src, size_t n) {
    size_t len = strlen(src);
    if (n > 0) {
        size_t copy = len < n - 1 ? len : n - 1;
        memcpy(dst, src, copy);
        dst[copy] = 0;
    }
    return len;
}

int wcscoll(const wchar_t *a, const wchar_t *b) {
    while (*a && *a == *b) {
        a++;
        b++;
    }
    return (int)*a - (int)*b;
}

size_t wcsxfrm(wchar_t *dst, const wchar_t *src, size_t n) {
    size_t len = wcslen(src);
    if (n > 0) {
        size_t copy = len < n - 1 ? len : n - 1;
        wmemcpy(dst, src, copy);
        dst[copy] = 0;
    }
    return len;
}

size_t wcsftime(wchar_t *dst, size_t n, const wchar_t *fmt, const struct tm *tm) {
    (void)fmt;
    (void)tm;
    if (n > 0) {
        dst[0] = 0;
    }
    return 0;
}

size_t wcrtomb(char *dst, wchar_t wc, mbstate_t *ps) {
    (void)ps;
    if (dst == NULL) {
        return 1;
    }
    dst[0] = (wc < 128) ? (char)wc : '?';
    return 1;
}

int getc(FILE *stream) { return fgetc(stream); }

wint_t getwc(FILE *stream) {
    (void)stream;
    return WEOF;
}

wint_t putwc(wchar_t wc, FILE *stream) {
    (void)wc;
    (void)stream;
    return WEOF;
}

wint_t ungetwc(wint_t wc, FILE *stream) {
    (void)wc;
    (void)stream;
    return WEOF;
}
