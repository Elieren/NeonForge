#![no_std]
#![no_main]

use core::panic::PanicInfo;
use x86_64::instructions::port::Port;
mod commands;
mod constants;
mod eng;
mod vga;

use crate::eng::SCANCODE_MAP;
use constants::{COLS, CURRENT_COL, CURRENT_ROW, MAX_LINES, MSG, ROWS};

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

#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe {
        let screen_width = 80;
        let screen_height = 25;
        vga::clear_screen(screen_width, screen_height);
        print_centered(ASCII_LOGO, screen_width, screen_height);
        print_loading_animation(
            screen_height as usize - 1 - 8,
            screen_width as usize / 2 - 1,
        );

        // delay(100000000);

        loop {
            if let Some(_) = get_key() {
                break;
            }
        }

        // delay(1000000);

        vga::clear_screen(screen_width, screen_height);
        CURRENT_COL = print_prompt(CURRENT_ROW, CURRENT_COL);

        loop {
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
        if scancode != LAST_SCANCODE {
            LAST_SCANCODE = scancode;
            delay(100000);
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
            }
        } else if let Some(character) = SCANCODE_MAP[key as usize] {
            if character == '\n' {
                // Выполнение команды и отображение текущей строки
                let stat: bool = commands::command_fn(&raw mut BUFFER, CURRENT_ROW);
                if !stat {
                    CURRENT_ROW += 2;
                } else {
                    CURRENT_ROW = 0;
                }
                CURRENT_COL = 0;

                // Печать приглашения
                CURRENT_COL = print_prompt(CURRENT_ROW, CURRENT_COL);
            } else {
                if CURRENT_COL < COLS {
                    if CURRENT_COL > 78 {
                        CURRENT_COL = 0;
                        CURRENT_ROW += 1;
                    }
                    BUFFER[CURRENT_ROW][CURRENT_COL] = character as u8;
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
