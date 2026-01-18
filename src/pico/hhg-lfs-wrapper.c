/***************************************************************************
 *
 * Hi Happy Garden
 * Copyright (C) 2023/2026 Antonio Salsi <passy.linux@zresa.it>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 ***************************************************************************/

#include "FreeRTOSConfig.h"

#include <hardware/flash.h>
#include <hardware/sync.h>
#include <pico/types.h>
#include <lfs.h>
#include <FreeRTOS.h>
#include <semphr.h>



#define HHG_FS_SIZE  (FLASH_PAGE_SIZE * 1024) // 256KB for the filesystem


static int flash_read(const struct lfs_config *c, lfs_block_t block, lfs_off_t off, void *buffer, lfs_size_t size);

static int flash_prog(const struct lfs_config *c, lfs_block_t block, lfs_off_t off, const void *buffer, lfs_size_t size);

static int flash_erase(const struct lfs_config *c, lfs_block_t block);

static int flash_sync(const struct lfs_config *c);

static int flash_lock(void);

static int flash_unlock(void);

static const struct lfs_config pico_cfg = {
    // block device operations
    .read  = flash_read,
    .prog  = flash_prog,
    .erase = flash_erase,
    .sync  = flash_sync,

    // block device configuration
    .read_size = 1,
    .prog_size = FLASH_PAGE_SIZE,
    .block_size = FLASH_SECTOR_SIZE,
    .block_count =  HHG_FS_SIZE / FLASH_SECTOR_SIZE,
    .cache_size = FLASH_SECTOR_SIZE / 4,
    .lookahead_size = 32,
    .block_cycles = 500,
};

static SemaphoreHandle_t mutex;
static lfs_t lfs;

const char* HHG_FS_BASE = (char*)(PICO_FLASH_SIZE_BYTES - HHG_FS_SIZE);


static int flash_read(const struct lfs_config *c, lfs_block_t block, lfs_off_t off, void *buffer, lfs_size_t size)
{
    assert(block < pico_cfg.block_count);
    assert(off + size <= pico_cfg.block_size);
    // read flash via XIP mapped space
    memcpy(buffer, HHG_FS_BASE + XIP_NOCACHE_NOALLOC_BASE + (block * pico_cfg.block_size) + off, size);
    return LFS_ERR_OK;
}

int flash_prog(const struct lfs_config *c, lfs_block_t block, lfs_off_t off, const void *buffer, lfs_size_t size) {
    assert(block < pico_cfg.block_count);
    // program with SDK
    uint32_t p = (uint32_t)HHG_FS_BASE + (block * pico_cfg.block_size) + off;
    uint32_t ints = save_and_disable_interrupts();
    flash_range_program(p, buffer, size);
    restore_interrupts(ints);
    return LFS_ERR_OK;
}


int flash_erase(const struct lfs_config *c, lfs_block_t block) {
    assert(block < pico_cfg.block_count);
    // erase with SDK
    uint32_t p = (uint32_t)HHG_FS_BASE + block * pico_cfg.block_size;
    uint32_t ints = save_and_disable_interrupts();
    flash_range_erase(p, pico_cfg.block_size);
    restore_interrupts(ints);
    return LFS_ERR_OK;
}

int flash_sync(const struct lfs_config *c) {
    return LFS_ERR_OK;
}

int flash_lock(void) {    
    return xSemaphoreTakeRecursive(mutex,  portMAX_DELAY) == pdTRUE ? LFS_ERR_OK : LFS_ERR_IO;
}

int flash_unlock(void) {
    return xSemaphoreGiveRecursive(mutex) == pdTRUE ? LFS_ERR_OK : LFS_ERR_IO;
}

/// public functions

int hhg_flash_mount(bool format) {

    mutex = xSemaphoreCreateRecursiveMutex();
    if( mutex == NULL )
    {
        return LFS_ERR_IO;
    }

    if (format) {
        lfs_format(&lfs, &pico_cfg);
    }
        
    // mount the filesystem
    return lfs_mount(&lfs, &pico_cfg);
}

long hhg_flash_open(const char* path, int flags) {
    lfs_file_t* file = lfs_malloc(sizeof(lfs_file_t));
    if (file == NULL) {
        return LFS_ERR_NOMEM;
    }
        
    int err = lfs_file_open(&lfs, file, path, flags);
    if (err != LFS_ERR_OK) {
        lfs_free(file);
        return err;
    }
    return (long)file;
}

int hhg_flash_close(long file) {
    int res = lfs_file_close(&lfs, (lfs_file_t*)file);
    lfs_free((lfs_file_t*)file);
    return res;
}

lfs_size_t hhg_flash_write(long file, const void* buffer, lfs_size_t size) {
    return lfs_file_write(&lfs, (lfs_file_t*)file, buffer, size);
}

lfs_size_t hhg_flash_read(long file, void* buffer, lfs_size_t size) {
    return lfs_file_read(&lfs, (lfs_file_t*)file, buffer, size);
}

int hhg_flash_rewind(long file) { 
    return lfs_file_rewind(&lfs, (lfs_file_t*)file); 
}

int hhg_flash_umount() {
    int res = lfs_unmount(&lfs);
    vSemaphoreDelete(mutex);
    return res;
}

int hhg_flash_remove(const char* path) {
    return lfs_remove(&lfs, path);
}

int hhg_flash_rename(const char* oldpath, const char* newpath) { 
    return lfs_rename(&lfs, oldpath, newpath); 
}

int hhg_flash_fsstat(lfs_size_t* block_size, lfs_size_t* block_count, lfs_size_t* blocks_used) { 
    *block_size = pico_cfg.block_size;
    *block_count = pico_cfg.block_count;
    *blocks_used = lfs_fs_size(&lfs);
    return LFS_ERR_OK;
}

lfs_soff_t hhg_flash_lseek(int file, lfs_soff_t off, int whence) {
    return lfs_file_seek(&lfs, (lfs_file_t*)file, off, whence);
}


int hhg_flash_truncate(int file, lfs_off_t size) { 
    return lfs_file_truncate(&lfs, (lfs_file_t*)file, size); 
}

lfs_soff_t hhg_flash_tell(int file) { 
    return lfs_file_tell(&lfs, (lfs_file_t*)file); 
}

int hhg_flash_stat(const char* path, uint8_t* type, lfs_size_t* size, char* name) { 

    struct lfs_info info;
    int res = lfs_stat(&lfs, path, &info);
    if (res != LFS_ERR_OK) {
        return res;
    }

    *type = info.type;
    *size = info.size;
    strcpy(name, info.name);

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


int hhg_flash_fflush(int file) { 
    return lfs_file_sync(&lfs, (lfs_file_t*)file); 
}

lfs_soff_t hhg_flash_size(int file) { 
    return lfs_file_size(&lfs, (lfs_file_t*)file); 
}

int hhg_flash_mkdir(const char* path) { 
    return lfs_mkdir(&lfs, path); 
}

long hhg_flash_dir_open(const char* path) {
	lfs_dir_t* dir = lfs_malloc(sizeof(lfs_dir_t));
	if (dir == NULL) {
        return -1;
    }
		
	if (lfs_dir_open(&lfs, dir, path) != LFS_ERR_OK) {
		lfs_free(dir);
		return -1;
	}
	return (long)dir;
}

int hhg_flash_dir_close(long dir) {
	int res = lfs_dir_close(&lfs, (lfs_dir_t*)dir);
	lfs_free((void*)dir);
    return res;
}

int hhg_flash_dir_read(long dir, uint8_t* type, lfs_size_t* size, char* name) { 

    struct lfs_info info;
    int res = lfs_dir_read(&lfs, (lfs_dir_t*)dir, &info);
    if (res != LFS_ERR_OK) {
        return res;
    }
    *type = info.type;
    *size = info.size;
    strcpy(name, info.name);
    return LFS_ERR_OK;
}

int hhg_flash_dir_seek(long dir, lfs_off_t off) { 
    return lfs_dir_seek(&lfs, (lfs_dir_t*)dir, off); 
}

lfs_soff_t hhg_flash_dir_tell(long dir) { 
    return lfs_dir_tell(&lfs, (lfs_dir_t*)dir); 
}

int hhg_flash_dir_rewind(long dir) { 
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

    for (int i = 0; i < sizeof(mesgs) / sizeof(mesgs[0]); i++)
        if (err == mesgs[i].err)
            return mesgs[i].text;
    return "Unknown error";
}
