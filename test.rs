#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(asm)]

use core::panic::PanicInfo;
mod commands;
mod constants;
mod eng;
mod file_system;
mod time;
mod vga;

use constants::{COLS, CURRENT_COL, CURRENT_ROW, ROWS};
use time::{enable_interrupts, init_idt, init_pit, set_time};

static ASCII_LOGO: &[u8] = b"
    ____  ____  _____
   / __ \\/ __ \\/ ___/
  / / / / / / /\\__ \\ 
 / /_/ / /_/ /___/ / 
/_____\\/____//____/
";

static mut BUFFER: [[u8; COLS]; ROWS] = [[0; COLS]; ROWS];
static mut CURSOR_POSITION_ROW: usize = 0;
static mut CURSOR_POSITION_COL: usize = 0;

unsafe fn inb(port: u16) -> u8 {
    let result: u8;
    asm!("inb %dx, %al", out("al") result, in("dx") port);
    result
}

unsafe fn inw(port: u16) -> u16 {
    let result: u16;
    asm!("inw %dx, %ax", out("ax") result, in("dx") port);
    result
}

const SECTOR_SIZE: usize = 512;
const FAT_SECTORS: usize = 9; // Примерное значение, нужно настроить в соответствии с вашей файловой системой.
const FAT_START_SECTOR: u64 = 1; // Начальный сектор таблицы FAT на диске.

unsafe fn read_sector(sector: u64, buffer: &mut [u8; SECTOR_SIZE]) {
    let lba = sector as u32;
    let drive = 0x1F0;

    asm!(
        "out dx, al",
        in("al") 0xE0 | ((lba >> 24) & 0x0F) as u8,
        in("dx") 0x1F6
    );
    asm!(
        "out dx, al",
        in("al") 1u8,
        in("dx") 0x1F2
    );
    asm!(
        "out dx, al",
        in("al") (lba & 0xFF) as u8,
        in("dx") 0x1F3
    );
    asm!(
        "out dx, al",
        in("al") ((lba >> 8) & 0xFF) as u8,
        in("dx") 0x1F4
    );
    asm!(
        "out dx, al",
        in("al") ((lba >> 16) & 0xFF) as u8,
        in("dx") 0x1F5
    );
    asm!(
        "out dx, al",
        in("al") 0x20u8,
        in("dx") 0x1F7
    );

    while (inb(0x1F7) & 0x80) != 0 {}

    for i in 0..SECTOR_SIZE / 2 {
        let data = inw(drive);
        buffer[i * 2] = data as u8;
        buffer[i * 2 + 1] = (data >> 8) as u8;
    }
}

pub fn initialize_filesystem() -> Result<(), &'static str> {
    let mut boot_sector = [0u8; SECTOR_SIZE];
    let fat_table = [0u8; SECTOR_SIZE * FAT_SECTORS];

    // Чтение загрузочного сектора
    unsafe {
        read_sector(0, &mut boot_sector);
    }

    // Чтение таблицы FAT
    for i in 0..FAT_SECTORS {
        unsafe {
            read_sector(FAT_START_SECTOR + i as u64, &mut fat_table[i * SECTOR_SIZE..(i + 1) * SECTOR_SIZE].try_into().unwrap());
        }
    }

    Ok(())
}

pub fn calculate_free_space() -> u64 {
    let mut free_clusters = 0;
    let fat_start_sector = FAT_START_SECTOR;
    let fat_size_sectors = FAT_SECTORS;
    let mut buffer = [0u8; SECTOR_SIZE];

    for sector in 0..fat_size_sectors {
        unsafe {
            read_sector(fat_start_sector + sector as u64, &mut buffer);
        }
        for i in 0..(SECTOR_SIZE / 2) {
            let cluster_entry = u16::from_le_bytes([buffer[i * 2], buffer[i * 2 + 1]]);
            if cluster_entry == 0 {
                free_clusters += 1;
            }
        }
    }

    let cluster_size = 4096; // Пример для 4 KB кластеров
    free_clusters as u64 * cluster_size
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    init_idt();
    init_pit();
    enable_interrupts();

    set_time(12, 0, 0); // Установка начального времени

    if let Err(err) = initialize_filesystem() {
        panic!("Ошибка инициализации файловой системы: {}", err);
    }

    unsafe {
        let screen_width = 80;
        let screen_height = 25;
        vga::clear_screen(screen_width, screen_height);
        print_centered(ASCII_LOGO, screen_width, screen_height);
        print_loading_animation(
            screen_height as usize - 1 - 8,
            screen_width as usize / 2 - 1,
        );

        vga::clear_screen(screen_width, screen_height);
        CURRENT_COL = print_prompt(CURRENT_ROW, CURRENT_COL);

        CURSOR_POSITION_COL = CURRENT_COL;

        // Отображение курсора на текущей позиции
        let cursor_row = CURSOR_POSITION_ROW;
        let cursor_col = CURSOR_POSITION_COL;
        let vga_buffer = 0xb8000 as *mut u8;
        *vga_buffer.offset((cursor_row as isize * COLS as isize + cursor_col as isize) * 2) = b'_';
        *vga_buffer.offset((cursor_row as isize * COLS as isize + cursor_col as isize) * 2 + 1) = 0x07;

        // Получить свободное пространство
        let free_space = calculate_free_space();
        
        // Преобразование числа в строку и вывод в центр экрана
        let mut num_buffer = [0u8; 20]; // Буфер для хранения строки числа
        let len = num_to_str(free_space, &mut num_buffer);
        
        // Подготовить сообщение
        let message = b"Свободное пространство: ";
        let message_len = message.len();

        // Вывести сообщение и значение в центр экрана
        let total_len = message_len + len;
        let start_col = (screen_width - total_len) / 2;
        let start_row = screen_height / 2;

        for (i, &byte) in message.iter().enumerate() {
            *vga_buffer.offset(((start_row * COLS + start_col + i) * 2) as isize) = byte;
            *vga_buffer.offset(((start_row * COLS + start_col + i) * 2 + 1) as isize) = 0x07; // Атрибуты цвета
        }

        for (i, &byte) in num_buffer.iter().take(len).enumerate() {
            *vga_buffer.offset(((start_row * COLS + start_col + message_len + i) * 2) as isize) = byte;
            *vga_buffer.offset(((start_row * COLS + start_col + message_len + i) * 2 + 1) as isize) = 0x07; // Атрибуты цвета
        }

        loop {
            if let Some(key) = get_key() {
                print_key(key, screen_width, screen_height);
            }
        }
    }
}

fn num_to_str(mut num: u64, buffer: &mut [u8]) -> usize {
    if num == 0 {
        buffer[0] = b'0';
        return 1;
    }

    let mut len = 0;
    while num > 0 {
        buffer[len] = (num % 10) as u8 + b'0';
        num /= 10;
        len += 1;
    }
    buffer[..len].reverse();
    len
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
