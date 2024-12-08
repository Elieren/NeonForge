use crate::constants::{COLS, CURRENT_COL, CURRENT_ROW, ROWS};
use crate::datetime::{get_date, get_time, set_date, set_time};
use crate::vga::{clear_screen, write_char};
use core::arch::asm;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

struct Command<'a> {
    name: &'a str,
    action: fn(*mut [[u8; COLS]; ROWS], usize) -> bool,
}

impl<'a> Command<'a> {
    fn new(name: &'a str, action: fn(*mut [[u8; COLS]; ROWS], usize) -> bool) -> Self {
        Command { name, action }
    }
}

fn hello_action(buffer: *mut [[u8; COLS]; ROWS], row: usize) -> bool {
    unsafe {
        let msg = b"HELLO!";
        for (i, &byte) in msg.iter().enumerate() {
            write_char(row + 1, i, byte, 0x07); // Печатает на строке row + 1
            (*buffer)[row + 1][i] = byte; // Записываем в буфер
        }
        false
    }
}

fn time_action(buffer: *mut [[u8; COLS]; ROWS], row: usize) -> bool {
    unsafe {
        let time = get_time();
        let time_str = format!("{:02}:{:02}:{:02}", time.0, time.1, time.2);

        for (i, byte) in time_str.bytes().enumerate() {
            write_char(row + 1, i, byte, 0x07); // Печатает на строке row + 1
            (*buffer)[row + 1][i] = byte; // Записываем в буфер
        }
        false
    }
}

fn date_action(buffer: *mut [[u8; COLS]; ROWS], row: usize) -> bool {
    unsafe {
        let date = get_date();
        let date_str = format!("{:02}.{:02}.{:04}", date.0, date.1, date.2);

        for (i, byte) in date_str.bytes().enumerate() {
            write_char(row + 1, i, byte, 0x07); // Печатает на строке row + 1
            (*buffer)[row + 1][i] = byte; // Записываем в буфер
        }
        false
    }
}

fn date_set_action(buffer: *mut [[u8; COLS]; ROWS], row: usize) -> bool {
    unsafe {
        let command: &[u8] = &(*buffer)[row][12..22]; // Извлекаем аргументы после `time_add`

        let command_str = core::str::from_utf8(command).unwrap_or("").trim();

        let mut parts = command_str.split('.');

        if let (Some(d), Some(m), Some(y)) = (parts.next(), parts.next(), parts.next()) {
            if let (Ok(day), Ok(month), Ok(year)) =
                (d.parse::<u8>(), m.parse::<u8>(), y.parse::<u16>())
            {
                set_date(day, month, year);
                let msg = b"Date set!";
                for (i, &byte) in msg.iter().enumerate() {
                    write_char(row + 1, i, byte, 0x07); // Печатает на строке row + 1
                    (*buffer)[row + 1][i] = byte; // Записываем в буфер
                }
                return false;
            }
        }

        let msg = b"Invalid date format!";
        for (i, &byte) in msg.iter().enumerate() {
            write_char(row + 1, i, byte, 0x07); // Печатает на строке row + 1
            (*buffer)[row + 1][i] = byte; // Записываем в буфер
        }
        false
    }
}

fn time_set_action(buffer: *mut [[u8; COLS]; ROWS], row: usize) -> bool {
    unsafe {
        let command: &[u8] = &(*buffer)[row][12..20]; // Извлекаем аргументы после `time_add`

        let command_str = core::str::from_utf8(command).unwrap_or("").trim();

        let mut parts = command_str.split(':');

        if let (Some(h), Some(m), Some(s)) = (parts.next(), parts.next(), parts.next()) {
            if let (Ok(hours), Ok(minutes), Ok(seconds)) =
                (h.parse::<u8>(), m.parse::<u8>(), s.parse::<u8>())
            {
                set_time(hours, minutes, seconds);
                let msg = b"Time set!";
                for (i, &byte) in msg.iter().enumerate() {
                    write_char(row + 1, i, byte, 0x07); // Печатает на строке row + 1
                    (*buffer)[row + 1][i] = byte; // Записываем в буфер
                }
                return false;
            }
        }

        let msg = b"Invalid time format!";
        for (i, &byte) in msg.iter().enumerate() {
            write_char(row + 1, i, byte, 0x07); // Печатает на строке row + 1
            (*buffer)[row + 1][i] = byte; // Записываем в буфер
        }
        false
    }
}

fn error_action(buffer: *mut [[u8; COLS]; ROWS], row: usize) -> bool {
    unsafe {
        let msg = b"Error: command";
        for (i, &byte) in msg.iter().enumerate() {
            write_char(row + 1, i, byte, 0x07); // Печатает на строке row + 1
            (*buffer)[row + 1][i] = byte; // Записываем в буфер
        }
        false
    }
}

fn reboot_action(buffer: *mut [[u8; COLS]; ROWS], row: usize) -> bool {
    unsafe {
        let msg = b"Rebooting...";
        for (i, &byte) in msg.iter().enumerate() {
            write_char(row + 1, i, byte, 0x07); // Печатает на строке row + 1
            (*buffer)[row + 1][i] = byte; // Записываем в буфер
        }

        asm!(
            "cli",            // Отключаем прерывания
            "out 0x64, al",   // Отправляем команду на контроллер клавиатуры
            "2: hlt",         // Метка 2: останавливаем процессор
            "jmp 2b",         // Переход к метке 2, чтобы создать бесконечный цикл
            in("al") 0xFEu8   // Значение 0xFE для команды перезагрузки
        );
        false
    }
}

fn shutdown_action(buffer: *mut [[u8; COLS]; ROWS], row: usize) -> bool {
    unsafe {
        let msg = b"Shutting down...";
        for (i, &byte) in msg.iter().enumerate() {
            write_char(row + 1, i, byte, 0x07); // Печатает на строке row + 1
            (*buffer)[row + 1][i] = byte; // Записываем в буфер
        }

        asm!(
            "cli",            // Отключаем прерывания
            "mov ax, 0x5301", // Подключаемся к APM API
            "xor bx, bx",
            "int 0x15",
            "mov ax, 0x530E", // Устанавливаем версию APM на 1.2
            "xor bx, bx",
            "mov cx, 0x0102",
            "int 0x15",
            "mov ax, 0x5307", // Выключаем систему
            "mov bx, 0x0001",
            "mov cx, 0x0003",
            "int 0x15",
            "hlt", // Останавливаем процессор
            options(noreturn, nostack)
        );
    }
}

fn clear(buffer: *mut [[u8; COLS]; ROWS], _: usize) -> bool {
    let screen_width = 80;
    let screen_height = 25;
    clear_screen(screen_width, screen_height);

    unsafe {
        for row in (*buffer).iter_mut() {
            for cell in row.iter_mut() {
                *cell = 0;
            }
        }
        CURRENT_COL = 0;
        CURRENT_ROW = 0;
    }

    true // Возвращаем true
}

pub fn command_fn(buffer: *mut [[u8; COLS]; ROWS], row: usize, command: &String) -> bool {
    let (cmd, _) = match command.find(' ') {
        Some(pos) => command.split_at(pos),
        None => (command.as_str(), ""),
    };

    let comm = cmd.trim();

    // Фильтруем только непустые и ненулевые байты
    let mut comm_filtered: Vec<u8> = Vec::new();
    for &byte in comm.as_bytes().iter() {
        if byte != 0 && !byte.is_ascii_whitespace() {
            comm_filtered.push(byte);
        }
    }

    let commands: [Command; 9] = [
        Command::new("hello", hello_action),
        Command::new("time", time_action),
        Command::new("time_set", time_set_action),
        Command::new("date", date_action),
        Command::new("date_set", date_set_action),
        Command::new("error", error_action),
        Command::new("reboot", reboot_action),
        Command::new("shutdown", shutdown_action),
        Command::new("clear", clear),
    ];

    for cmd in commands.iter() {
        let cmd_name_bytes: Vec<u8> = cmd.name.bytes().collect();

        if comm_filtered == cmd_name_bytes {
            let result = (cmd.action)(buffer, row);
            if result {
                return true;
            }
            return false; // Завершите цикл, если команда найдена, но не вернула true
        }
    }

    error_action(buffer, row);
    false // Возвращаем false, если команда не найдена
}
