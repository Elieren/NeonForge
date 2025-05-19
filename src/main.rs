#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(global_allocator)]
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use core::panic::PanicInfo;
use x86_64::instructions::port::Port;
mod commands;
mod constants;
mod datetime;
mod eng;
// mod file_system;
mod gpio;
mod interrupts;
mod pic;
mod pit;
mod vga;

use core::mem::MaybeUninit;
use linked_list_allocator::LockedHeap;

use crate::eng::SCANCODE_MAP;
use constants::{
    COLOR_INFO, COLS, CURRENT_COL, CURRENT_ROW, HEAP_SIZE, MAX_LINES, MSG, PARTITION_OFFSET, ROWS,
};
use datetime::{get_date, get_time};
use interrupts::{enable_interrupts, init_idt};
use pit::init_pit;

use core::ptr::NonNull;
use embedded_sdmmc::{Controller, Mode, VolumeIdx};
// use file_system::MyBlockDevice;

use gpio::Gpio;
use vga::{write_char, write_string};

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

fn init_heap() {
    static mut HEAP_MEMORY: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        let heap_start = HEAP_MEMORY.as_mut_ptr() as *mut u8;
        ALLOCATOR.lock().init(heap_start, HEAP_SIZE);
    }
}

struct MyTimeSource;

impl embedded_sdmmc::TimeSource for MyTimeSource {
    fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
        embedded_sdmmc::Timestamp {
            year_since_1970: 52,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}

static ASCII_LOGO: &[u8] = br#"
                                         
 _____             _____                 
|   | |___ ___ ___|   __|___ ___ ___ ___ 
| | | | -_| . |   |   __| . |  _| . | -_|
|_|___|___|___|_|_|__|  |___|_| |_  |___|
                                |___|    
"#;

static mut BUFFER: [[u8; COLS]; ROWS] = [[0; COLS]; ROWS];
static mut CURSOR_POSITION_ROW: usize = 0;
static mut CURSOR_POSITION_COL: usize = 0;
static mut INPUT_BUFFER: String = String::new();

#[no_mangle]
pub extern "C" fn _start() -> ! {
    write_string(0, 0, "Initializing heap...", 0x0F);
    init_heap();
    write_string(1, 0, "Heap initialized.", 0x0F);

    init_idt();
    init_pit();
    enable_interrupts();

    // Инициализация блокового устройства
    // write_string(2, 0, "Initializing block device...", 0x0F);
    // let base_address = NonNull::new((0x100000 + PARTITION_OFFSET) as *mut u8).unwrap();
    // let size = 5 * 1024 * 1024 * 1024; // Размер диска 5 ГБ
    // let block_device = MyBlockDevice::new(base_address, size);
    // write_string(3, 0, "Block device initialized.", 0x0F);

    // write_string(4, 0, "Initializing FAT controller...", 0x0F);
    // let mut controller = Controller::new(block_device, MyTimeSource);
    // write_string(5, 0, "FAT controller initialized.", 0x0F);

    // write_string(6, 0, "Mounting volume...", 0x0F);
    // match controller.get_volume(VolumeIdx(0)) {
    //     Ok(mut volume) => {
    //         write_string(7, 0, "Volume mounted.", 0x0F);
    //         let root_dir = controller.open_root_dir(&volume).unwrap();

    //         write_string(8, 0, "Writing data to file...", 0x0F);
    //         let data = b"Hello, world!";
    //         let mut file = controller
    //             .open_file_in_dir(&mut volume, &root_dir, "example.txt", Mode::ReadWriteCreate)
    //             .unwrap();
    //         controller.write(&mut volume, &mut file, data).unwrap();
    //         controller.close_file(&volume, file).unwrap();
    //         write_string(9, 0, "Data written to file.", 0x0F);

    //         write_string(10, 0, "Reading data from file...", 0x0F);
    //         let mut read_file = controller
    //             .open_file_in_dir(&mut volume, &root_dir, "example.txt", Mode::ReadOnly)
    //             .unwrap();
    //         let mut buffer = [0u8; 13];
    //         controller
    //             .read(&volume, &mut read_file, &mut buffer)
    //             .unwrap();
    //         controller.close_file(&volume, read_file).unwrap();
    //         write_string(11, 0, "Data read from file.", 0x0F);

    //         let content_str = core::str::from_utf8(&buffer).unwrap();
    //         write_string(12, 0, "File content: ", 0x0F);
    //         write_string(13, 0, content_str, 0x0F);
    //     }
    //     Err(e) => {
    //         write_string(7, 0, "Failed to mount volume.", 0x4F);
    //         write_string(8, 0, &format!("Error: {:?}", e), 0x4F);
    //     }
    // }

    // delay(100000000);

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
        *vga_buffer.offset((cursor_row as isize * COLS as isize + cursor_col as isize) * 2 + 1) =
            0x07;

        loop {
            scroll_status();
            date_status();
            time_status();
            if let Some(key) = get_key() {
                print_key(key, screen_width, screen_height);
            }
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[alloc_error_handler]
fn alloc_error_handler(_layout: core::alloc::Layout) -> ! {
    loop {}
}

fn print_centered(msg: &[u8], width: u16, height: u16) {
    let mut lines: [&[u8]; MAX_LINES] = [&[]; MAX_LINES]; // Инициализация массива строк
    let mut line_count = 0;

    // Разделяем сообщение на строки и сохраняем их в массиве
    for line in msg.split(|&byte| byte == b'\n') {
        if line_count < MAX_LINES {
            lines[line_count] = line;
            line_count += 1;
        }
    }

    let start_row = height / 2 - (line_count as u16 / 2); // Начальная строка для центровки

    for (line_index, line) in lines.iter().enumerate().take(line_count) {
        let line_len = line.len() as u16;
        let start_col = (width / 2).saturating_sub(line_len / 2); // Начальная колонка для центровки
        let row = start_row + line_index as u16; // Текущая строка

        for (i, &byte) in line.iter().enumerate() {
            let vga_buffer = 0xb8000 as *mut u8;
            let offset = (row as isize * width as isize + start_col as isize + i as isize) * 2;
            unsafe {
                *vga_buffer.offset(offset) = byte; // Символ
                *vga_buffer.offset(offset + 1) = 0x0e; // Цвет символа (желтый)
            }
        }
    }
}

fn print_loading_animation(row: usize, col: usize) {
    let vga_buffer = 0xb8000 as *mut u8;
    let dots = [b'.', b'.', b'.'];

    for _ in 0..5 {
        // Количество повторов анимации
        for i in 0..dots.len() {
            // Зажигаем точки
            for j in 0..=i {
                unsafe {
                    *vga_buffer.offset((row as isize * 80 + (col + j) as isize) * 2) = dots[j];
                    *vga_buffer.offset((row as isize * 80 + (col + j) as isize) * 2 + 1) = 0x0e;
                    // Белый цвет
                }
            }

            delay(500000); // Задержка

            // Гасим точки
            for j in 0..=i {
                unsafe {
                    *vga_buffer.offset((row as isize * 80 + (col + j) as isize) * 2) = b' ';
                    *vga_buffer.offset((row as isize * 80 + (col + j) as isize) * 2 + 1) = 0x0e;
                    // Белый цвет
                }
            }
        }
    }
}

fn print_prompt(row: usize, col: usize) -> usize {
    unsafe {
        let msg = MSG;
        for (i, &byte) in msg.iter().enumerate() {
            BUFFER[row][col + i] = byte;
        }

        vga::print_buffer(&raw mut BUFFER);

        return col + msg.len();
    }
}

fn delay(a: u32) {
    for _ in 0..a {
        unsafe { core::ptr::read_volatile(&0) };
    }
}

fn get_key() -> Option<u8> {
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    static mut LAST_SCANCODE: u8 = 0;
    unsafe {
        if scancode == 0x0E {
            delay(200000);
            Some(scancode)
        } else if scancode != LAST_SCANCODE {
            LAST_SCANCODE = scancode;
            Some(scancode)
        } else {
            None
        }
    }
}

fn print_key(key: u8, width: u16, height: u16) {
    unsafe {
        if key == 0x0E {
            // Обработка Backspace
            if CURRENT_COL > MSG.len() {
                BUFFER[CURRENT_ROW][CURRENT_COL] = 0;
                CURRENT_COL -= 1;
                BUFFER[CURRENT_ROW][CURRENT_COL] = 0;
                INPUT_BUFFER.pop();
            }
        } else if let Some(character) = SCANCODE_MAP[key as usize] {
            if character == '\n' {
                // Выполнение команды и отображение текущей строки
                let stat: bool = commands::command_fn(&raw mut BUFFER, CURRENT_ROW, &INPUT_BUFFER);
                if !stat {
                    CURRENT_ROW += 2;
                }

                // Очистка буфера после выполнения команды
                INPUT_BUFFER.clear();

                CURRENT_COL = 0;
                if CURRENT_ROW >= 24 {
                    scroll();
                    CURRENT_ROW -= 1;
                    CURRENT_COL = 0;
                }
                // Печать приглашения
                CURRENT_COL = print_prompt(CURRENT_ROW, CURRENT_COL);
            } else {
                if CURRENT_COL < COLS {
                    if CURRENT_COL > 78 {
                        CURRENT_COL = 0;
                        CURRENT_ROW += 1;
                    }
                    BUFFER[CURRENT_ROW][CURRENT_COL] = character as u8;
                    INPUT_BUFFER.push(character);
                    CURRENT_COL += 1;
                }
            }
        }

        // Обновление текущей позиции курсора
        CURSOR_POSITION_ROW = CURRENT_ROW;
        CURSOR_POSITION_COL = CURRENT_COL;

        // Очищаем экран
        vga::clear_screen(width, height);

        // Печать буфера на экране
        vga::print_buffer(&raw mut BUFFER);

        // Отображение курсора на текущей позиции
        let cursor_row = CURSOR_POSITION_ROW;
        let cursor_col = CURSOR_POSITION_COL;
        let vga_buffer = 0xb8000 as *mut u8;
        *vga_buffer.offset((cursor_row as isize * width as isize + cursor_col as isize) * 2) = b'_';
        *vga_buffer.offset((cursor_row as isize * width as isize + cursor_col as isize) * 2 + 1) =
            0x07;
    }
}

fn scroll() {
    unsafe {
        for i in 0..24 {
            BUFFER[i] = BUFFER[i + 1];
        }
        BUFFER[24] = [0; COLS];
    }
}

fn scroll_status() {
    unsafe {
        if CURRENT_ROW == 24 {
            scroll();
            CURRENT_ROW -= 1;
            CURRENT_COL = 0;
            CURRENT_COL = print_prompt(CURRENT_ROW, CURRENT_COL);
        }
    }
}

fn time_status() {
    let time = get_time();
    let time_str = format!("{:02}:{:02}", time.0, time.1);

    for (i, byte) in time_str.bytes().enumerate() {
        write_char(24, i + 74, byte, COLOR_INFO); // Печатает на строке row + 1
    }
}

fn date_status() {
    let date = get_date();
    let date_str = format!("{:02}.{:02}.{:04}", date.0, date.1, date.2);

    for (i, byte) in date_str.bytes().enumerate() {
        write_char(24, i + 62, byte, COLOR_INFO); // Печатает на строке row + 1
    }
}
