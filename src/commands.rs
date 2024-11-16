use crate::constants::{COLS, ROWS};
use crate::time::{get_time, set_time};
use crate::vga::{clear_screen, write_char};
use core::arch::asm;

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
        let mut time_str = [0u8; 8];

        // Заполняем строку времени
        time_str[0] = b'0' + (time.hours / 10) % 10;
        time_str[1] = b'0' + time.hours % 10;
        time_str[2] = b':';
        time_str[3] = b'0' + (time.minutes / 10) % 10;
        time_str[4] = b'0' + time.minutes % 10;
        time_str[5] = b':';
        time_str[6] = b'0' + (time.seconds / 10) % 10;
        time_str[7] = b'0' + time.seconds % 10;

        for (i, &byte) in time_str.iter().enumerate() {
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
    }

    true // Возвращаем true
}

pub fn command_fn(buffer: *mut [[u8; COLS]; ROWS], row: usize) -> bool {
    unsafe {
        let command: &[u8] = &(*buffer)[row][3..];

        // Найти конец команды, игнорируя аргументы
        let (cmd, _) = match command.iter().position(|&byte| byte == b' ') {
            Some(pos) => command.split_at(pos),
            None => (command, &[][..]),
        };

        let comm = core::str::from_utf8(cmd).unwrap_or("").trim();

        // Фильтруем только непустые и ненулевые байты
        let mut comm_filtered: [u8; 256] = [0; 256];
        let mut index = 0;
        for &byte in comm.as_bytes().iter() {
            if byte != 0 && !byte.is_ascii_whitespace() {
                comm_filtered[index] = byte;
                index += 1;
            }
        }

        let commands: [Command; 7] = [
            Command::new("hello", hello_action),
            Command::new("time", time_action),
            Command::new("time_set", time_set_action),
            Command::new("error", error_action),
            Command::new("reboot", reboot_action),
            Command::new("shutdown", shutdown_action),
            Command::new("clear", clear),
        ];

        for cmd in commands.iter() {
            let mut cmd_name_bytes = [0u8; 256];
            for (i, byte) in cmd.name.bytes().enumerate() {
                cmd_name_bytes[i] = byte;
            }

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
}

// fn number_to_ascii_bytes(number: usize) -> [u8; 20] {
//     let mut buffer = [b'0'; 20];
//     let mut i = 19;
//     let mut n = number;

//     if n == 0 {
//         buffer[i] = b'0';
//     } else {
//         while n > 0 {
//             buffer[i] = b'0' + (n % 10) as u8;
//             n /= 10;
//             i -= 1;
//         }
//     }

//     // Сдвигаем байты влево, чтобы убрать лишние нули
//     let shift = i + 1;
//     let mut j = 0;
//     while shift + j < 20 {
//         buffer[j] = buffer[shift + j];
//         j += 1;
//     }
//     buffer[j] = 0; // Завершающий нуль

//     buffer
// }
