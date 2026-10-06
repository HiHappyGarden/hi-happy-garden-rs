/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, see <https://www.gnu.org/licenses/>.
 *
 ***************************************************************************/

// Host twin of src/pico/hhg-lfs-wrapper.c: same littlefs release, same
// geometry and same hhg_flash_* API, so the firmware sees identical
// filesystem behaviour (error codes included). Only the block device
// changes: a RAM image of the flash partition, optionally mirrored to a file
// so the content survives an emulator restart.

#include <errno.h>
#include <fcntl.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <lfs.h>

// RP2350 flash geometry, as FLASH_PAGE_SIZE / FLASH_SECTOR_SIZE of the pico-sdk
#define HHG_FLASH_PAGE_SIZE   256
#define HHG_FLASH_SECTOR_SIZE 4096

#define HHG_FS_SIZE  (HHG_FLASH_PAGE_SIZE * 1024) // 256KB for the filesystem

static int flash_read(const struct lfs_config *c, lfs_block_t block, lfs_off_t off, void *buffer, lfs_size_t size);

static int flash_prog(const struct lfs_config *c, lfs_block_t block, lfs_off_t off, const void *buffer, lfs_size_t size);

static int flash_erase(const struct lfs_config *c, lfs_block_t block);

static int flash_sync(const struct lfs_config *c);

static const struct lfs_config hhg_lfs_cfg = {
    // context
    .context = NULL,

    // block device operations
    .read  = flash_read,
    .prog  = flash_prog,
    .erase = flash_erase,
    .sync  = flash_sync,

    // block device configuration
    .read_size = 1,
    .prog_size = HHG_FLASH_PAGE_SIZE,
    .block_size = HHG_FLASH_SECTOR_SIZE,
    .block_count =  HHG_FS_SIZE / HHG_FLASH_SECTOR_SIZE,
    .cache_size = HHG_FLASH_SECTOR_SIZE / 4,
    .lookahead_size = 32,
    .block_cycles = 500,
};

static lfs_t lfs;

// Erased NOR flash reads back as 0xFF: the image starts that way, like a blank chip
static uint8_t flash_image[HHG_FS_SIZE];
static bool flash_image_ready = false;

// Backing file of the image, -1 when the flash lives in RAM only
static int flash_fd = -1;

static void flash_image_init(void) {
    if (!flash_image_ready) {
        memset(flash_image, 0xFF, sizeof(flash_image));
        // littlefs reports through printf: line buffered, its messages stay
        // in order with the firmware log even when stdout is a file or pipe
        setvbuf(stdout, NULL, _IOLBF, 0);
        flash_image_ready = true;
    }
}

static int flash_persist(size_t offset, size_t size) {
    if (flash_fd < 0) {
        return LFS_ERR_OK;
    }
    if (pwrite(flash_fd, flash_image + offset, size, (off_t)offset) != (ssize_t)size) {
        return LFS_ERR_IO;
    }
    return LFS_ERR_OK;
}

static int flash_read(const struct lfs_config *c, lfs_block_t block, lfs_off_t off, void *buffer, lfs_size_t size) {
    if (block >= hhg_lfs_cfg.block_count || off + size > hhg_lfs_cfg.block_size) {
        return LFS_ERR_IO;
    }
    memcpy(buffer, flash_image + (block * hhg_lfs_cfg.block_size) + off, size);
    return LFS_ERR_OK;
}

static int flash_prog(const struct lfs_config *c, lfs_block_t block, lfs_off_t off, const void *buffer, lfs_size_t size) {
    if (block >= hhg_lfs_cfg.block_count || off + size > hhg_lfs_cfg.block_size) {
        return LFS_ERR_IO;
    }
    size_t offset = (block * hhg_lfs_cfg.block_size) + off;
    // NOR programming can only clear bits: AND keeps a missing erase visible
    // as corruption, as it would be on the real chip
    const uint8_t* src = buffer;
    for (lfs_size_t i = 0; i < size; i++) {
        flash_image[offset + i] &= src[i];
    }
    return flash_persist(offset, size);
}

static int flash_erase(const struct lfs_config *c, lfs_block_t block) {
    if (block >= hhg_lfs_cfg.block_count) {
        return LFS_ERR_IO;
    }
    size_t offset = block * hhg_lfs_cfg.block_size;
    memset(flash_image + offset, 0xFF, hhg_lfs_cfg.block_size);
    return flash_persist(offset, hhg_lfs_cfg.block_size);
}

static int flash_sync(const struct lfs_config *c) {
    if (flash_fd >= 0 && fsync(flash_fd) != 0) {
        return LFS_ERR_IO;
    }
    return LFS_ERR_OK;
}

/// emulator only functions

int hhg_emulator_flash_set_image(const char* path) {
    flash_image_init();

    int fd = open(path, O_RDWR | O_CREAT, 0644);
    if (fd < 0) {
        return -errno;
    }

    ssize_t len = read(fd, flash_image, sizeof(flash_image));
    if (len < 0) {
        int err = -errno;
        close(fd);
        return err;
    }

    // A new (or short) file is padded with erased flash, so littlefs finds a
    // blank chip and the firmware formats it as on a fresh board
    if ((size_t)len < sizeof(flash_image)) {
        memset(flash_image + len, 0xFF, sizeof(flash_image) - (size_t)len);
        if (pwrite(fd, flash_image, sizeof(flash_image), 0) != (ssize_t)sizeof(flash_image)) {
            int err = -errno;
            close(fd);
            return err;
        }
    }

    if (flash_fd >= 0) {
        close(flash_fd);
    }
    flash_fd = fd;
    return 0;
}

/// public functions

int hhg_flash_mount(bool format) {

    const char* hhg_flash_errmsg(int err);

    flash_image_init();

    int err = lfs_mount(&lfs, &hhg_lfs_cfg);
    if (err == LFS_ERR_OK) {
        return err;
    }

    if (format) {
        err = lfs_format(&lfs, &hhg_lfs_cfg);
    }

    if (err != LFS_ERR_OK) {
        printf("LFS format error: %s\r\n", hhg_flash_errmsg(err));
        return err;
    }

    err = lfs_mount(&lfs, &hhg_lfs_cfg);

    if (err != LFS_ERR_OK) {
        printf("LFS mount error: %s\r\n", hhg_flash_errmsg(err));
        return err;
    }

    return err;
}

void* hhg_flash_open(const char* path, int flags, int* err) {
    lfs_file_t* file = malloc(sizeof(lfs_file_t));
    if (file == NULL) {
        *err = LFS_ERR_NOMEM;
        return NULL;
    }

    *err = lfs_file_open(&lfs, file, path, flags);
    if (*err != LFS_ERR_OK) {
        free(file);
        return NULL;
    }
    return (void*)file;
}

int hhg_flash_close(void* file) {
    int res = lfs_file_close(&lfs, (lfs_file_t*)file);
    free(file);
    return res;
}

lfs_ssize_t hhg_flash_write(void* file, const void* buffer, lfs_size_t size) {
    // littlefs asserts on a file not opened for writing: report it as an error instead
    if ((((lfs_file_t*)file)->flags & LFS_O_WRONLY) != LFS_O_WRONLY) {
        return LFS_ERR_BADF;
    }
    return lfs_file_write(&lfs, (lfs_file_t*)file, buffer, size);
}

lfs_ssize_t hhg_flash_read(void* file, void* buffer, lfs_size_t size) {
    // littlefs asserts on a file not opened for reading: report it as an error instead
    if ((((lfs_file_t*)file)->flags & LFS_O_RDONLY) != LFS_O_RDONLY) {
        return LFS_ERR_BADF;
    }
    return lfs_file_read(&lfs, (lfs_file_t*)file, buffer, size);
}

int hhg_flash_rewind(void* file) {
    return lfs_file_rewind(&lfs, (lfs_file_t*)file);
}

int hhg_flash_umount() {
    return lfs_unmount(&lfs);
}

int hhg_flash_remove(const char* path) {
    return lfs_remove(&lfs, path);
}

int hhg_flash_rename(const char* oldpath, const char* newpath) {
    return lfs_rename(&lfs, oldpath, newpath);
}

int hhg_flash_fsstat(lfs_size_t* block_size, lfs_size_t* block_count, lfs_size_t* blocks_used) {
    *block_size = hhg_lfs_cfg.block_size;
    *block_count = hhg_lfs_cfg.block_count;
    *blocks_used = lfs_fs_size(&lfs);
    return LFS_ERR_OK;
}

lfs_soff_t hhg_flash_lseek(void* file, lfs_soff_t off, int whence) {
    return lfs_file_seek(&lfs, (lfs_file_t*)file, off, whence);
}

int hhg_flash_truncate(void* file, lfs_off_t size) {
    return lfs_file_truncate(&lfs, (lfs_file_t*)file, size);
}

lfs_soff_t hhg_flash_tell(void* file) {
    return lfs_file_tell(&lfs, (lfs_file_t*)file);
}

int hhg_flash_stat(const char* path, uint8_t* type, lfs_size_t* size, char* name) {

    struct lfs_info info;
    int res = lfs_stat(&lfs, path, &info);
    if (res < LFS_ERR_OK) {
        return res;
    }

    *type = info.type;
    *size = info.size;
    strncpy(name, info.name, 255);
    name[255] = '\0';

    return LFS_ERR_OK;
}

lfs_ssize_t hhg_flash_getattr(const char* path, uint8_t type, void* buffer, lfs_size_t size) {
    return lfs_getattr(&lfs, path, type, buffer, size);
}

int hhg_flash_setattr(const char* path, uint8_t type, const void* buffer, lfs_size_t size) {
    return lfs_setattr(&lfs, path, type, buffer, size);
}

int hhg_flash_removeattr(const char* path, uint8_t type) {
    return lfs_removeattr(&lfs, path, type);
}

int hhg_flash_fflush(void* file) {
    return lfs_file_sync(&lfs, (lfs_file_t*)file);
}

lfs_soff_t hhg_flash_size(void* file) {
    return lfs_file_size(&lfs, (lfs_file_t*)file);
}

int hhg_flash_mkdir(const char* path) {
    return lfs_mkdir(&lfs, path);
}

void* hhg_flash_dir_open(const char* path) {
    lfs_dir_t* dir = malloc(sizeof(lfs_dir_t));
    if (dir == NULL) {
        return NULL;
    }

    if (lfs_dir_open(&lfs, dir, path) != LFS_ERR_OK) {
        free(dir);
        return NULL;
    }
    return (void*)dir;
}

int hhg_flash_dir_close(void* dir) {
    int res = lfs_dir_close(&lfs, (lfs_dir_t*)dir);
    free(dir);
    return res;
}

int hhg_flash_dir_read(void* dir, uint8_t* type, lfs_size_t* size, char* name) {

    // At the end of the directory lfs_dir_read returns 0 without filling
    // info: zeroed, the caller gets an empty name instead of stack garbage
    struct lfs_info info = {0};
    int res = lfs_dir_read(&lfs, (lfs_dir_t*)dir, &info);
    if (res < LFS_ERR_OK) {
        return res;
    }
    *type = info.type;
    *size = info.size;
    strncpy(name, info.name, 255);
    name[255] = '\0';
    return LFS_ERR_OK;
}

int hhg_flash_dir_seek(void* dir, lfs_off_t off) {
    return lfs_dir_seek(&lfs, (lfs_dir_t*)dir, off);
}

lfs_soff_t hhg_flash_dir_tell(void* dir) {
    return lfs_dir_tell(&lfs, (lfs_dir_t*)dir);
}

int hhg_flash_dir_rewind(void* dir) {
    return lfs_dir_rewind(&lfs, (lfs_dir_t*)dir);
}

const char* hhg_flash_errmsg(int err) {
    static const struct {
        int err;
        char* text;
    } mesgs[] = {{LFS_ERR_OK, "No error"},
                 {LFS_ERR_IO, "Error during device operation"},
                 {LFS_ERR_CORRUPT, "Corrupted"},
                 {LFS_ERR_NOENT, "No directory entry"},
                 {LFS_ERR_EXIST, "Entry already exists"},
                 {LFS_ERR_NOTDIR, "Entry is not a dir"},
                 {LFS_ERR_ISDIR, "Entry is a dir"},
                 {LFS_ERR_NOTEMPTY, "Dir is not empty"},
                 {LFS_ERR_BADF, "Bad file number"},
                 {LFS_ERR_FBIG, "File too large"},
                 {LFS_ERR_INVAL, "Invalid parameter"},
                 {LFS_ERR_NOSPC, "No space left on device"},
                 {LFS_ERR_NOMEM, "No more memory available"},
                 {LFS_ERR_NOATTR, "No data/attr available"},
                 {LFS_ERR_NAMETOOLONG, "File name too long"}};

    for (size_t i = 0; i < sizeof(mesgs) / sizeof(mesgs[0]); i++)
        if (err == mesgs[i].err)
            return mesgs[i].text;
    return "Unknown error";
}
