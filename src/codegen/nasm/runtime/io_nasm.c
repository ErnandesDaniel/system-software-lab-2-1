#include <stdio.h>
#include <stdint.h>

// Wrapper around standard C fread
size_t fread_nasm(void *buf, size_t size, size_t count, FILE *stream) {
    return fread(buf, size, count, stream);
}

// Wrapper around standard C fseek
int fseek_nasm(FILE *stream, long offset, int whence) {
    return fseek(stream, offset, whence);
}

// Read a 32-bit little-endian integer from buf + offset
int32_t read_le32_nasm(const void *buf, int offset) {
    const uint8_t *p = (const uint8_t *)buf + offset;
    return (int32_t)(p[0] | (p[1] << 8) | (p[2] << 16) | (p[3] << 24));
}

// Read a 16-bit little-endian integer from buf + offset
int32_t read_le16_nasm(const void *buf, int offset) {
    const uint8_t *p = (const uint8_t *)buf + offset;
    return (int32_t)(p[0] | (p[1] << 8));
}

// Read a single byte from buf + offset
int32_t read_i8_nasm(const void *buf, int offset) {
    return (int32_t)((const uint8_t *)buf)[offset];
}

// Wrapper around standard C fwrite
size_t fwrite_nasm(const void *buf, size_t size, size_t count, FILE *stream) {
    return fwrite(buf, size, count, stream);
}
