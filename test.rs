#![allow(dead_code)]
use core::arch::asm;

const SECTOR_SIZE: usize = 512;
const FAT_SECTORS: usize = 9; // Примерное значение
const FAT_START_SECTOR: u64 = 1; // Начальный сектор таблицы FAT

unsafe fn inb(port: u16) -> u8 {
    let result: u8;
    asm!("in al, dx", out("al") result, in("dx") port);
    result
}

unsafe fn inw(port: u16) -> u16 {
    let result: u16;
    asm!("in ax, dx", out("ax") result, in("dx") port);
    result
}

unsafe fn read_sector(sector: u64, buffer: &mut [u8; SECTOR_SIZE]) {
    let lba = sector as u32;

    // Отправка команды чтения в LBA-режиме
    asm!(
        "out dx, al",
        in("al") 0xE0 | ((lba >> 24) & 0x0F) as u8,
        in("dx") 0x1F6
    );
    asm!("out dx, al", in("al") 1u8, in("dx") 0x1F2);
    asm!("out dx, al", in("al") (lba & 0xFF) as u8, in("dx") 0x1F3);
    asm!("out dx, al", in("al") ((lba >> 8) & 0xFF) as u8, in("dx") 0x1F4);
    asm!("out dx, al", in("al") ((lba >> 16) & 0xFF) as u8, in("dx") 0x1F5);
    asm!("out dx, al", in("al") 0x20u8, in("dx") 0x1F7);

    // Ожидание завершения команды
    while (inb(0x1F7) & 0x80) != 0 {}

    // Чтение данных из порта
    for i in 0..SECTOR_SIZE / 2 {
        let data = inw(0x1F0);
        buffer[i * 2] = data as u8;
        buffer[i * 2 + 1] = (data >> 8) as u8;
    }
}

pub fn initialize_filesystem() -> Result<(), &'static str> {
    let mut boot_sector = [0u8; SECTOR_SIZE];
    unsafe {
        read_sector(0, &mut boot_sector);
    }

    let bytes_per_sector = u16::from_le_bytes([boot_sector[11], boot_sector[12]]);
    let sectors_per_cluster = boot_sector[13];
    let reserved_sectors = u16::from_le_bytes([boot_sector[14], boot_sector[15]]);
    let num_fats = boot_sector[16];
    let fat_size_sectors = u16::from_le_bytes([boot_sector[22], boot_sector[23]]);

    if bytes_per_sector != SECTOR_SIZE as u16 {
        return Err("Unsupported sector size");
    }

    // Сохраните или используйте параметры для дальнейшей работы
    Ok(())
}

pub fn calculate_free_space() -> u64 {
    let mut free_clusters = 0;
    let mut buffer = [0u8; SECTOR_SIZE];

    for sector in FAT_START_SECTOR..(FAT_START_SECTOR + FAT_SECTORS as u64) {
        unsafe {
            read_sector(sector, &mut buffer);
        }
        for i in 0..(SECTOR_SIZE / 2) {
            let cluster_entry = u16::from_le_bytes([buffer[i * 2], buffer[i * 2 + 1]]);
            if cluster_entry == 0 {
                free_clusters += 1;
            }
        }
    }

    let cluster_size = SECTOR_SIZE * 4; // 4 сектора на кластер (пример)
    free_clusters as u64 * cluster_size as u64
}
