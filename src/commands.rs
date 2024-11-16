use crate::constants::{COLS, ROWS};
use crate::vga::write_char;
use core::arch::asm;

struct Command<'a> {
    name: &'a str,
    action: fn(&mut [[u8; COLS]; ROWS], usize),
}

impl<'a> Command<'a> {
    fn new(name: &'a str, action: fn(&mut [[u8; COLS]; ROWS], usize)) -> Self {
        Command { name, action }
    }
}

fn hello_action(buffer: &mut [[u8; COLS]; ROWS], row: usize) {
    let msg = b"HELLO!";
    for (i, &byte) in msg.iter().enumerate() {
        write_char(row + 1, i, byte, 0x07); // Печатает на строке row + 1
        buffer[row + 1][i] = byte; // Записываем в буфер
    }
}

fn error_action(buffer: &mut [[u8; COLS]; ROWS], row: usize) {
    let msg = b"Error: command";
    for (i, &byte) in msg.iter().enumerate() {
        write_char(row + 1, i, byte, 0x07); // Печатает на строке row + 1
        buffer[row + 1][i] = byte; // Записываем в буфер
    }
}

fn reboot_action(buffer: &mut [[u8; COLS]; ROWS], row: usize) {
    let msg = b"Rebooting...";
    for (i, &byte) in msg.iter().enumerate() {
        write_char(row + 1, i, byte, 0x07); // Печатает на строке row + 1
        buffer[row + 1][i] = byte; // Записываем в буфер
    }

    unsafe {
        asm!(
            "cli",            // Отключаем прерывания
            "out 0x64, al",   // Отправляем команду на контроллер клавиатуры
            "2: hlt",         // Метка 2: останавливаем процессор
            "jmp 2b",         // Переход к метке 2, чтобы создать бесконечный цикл
            in("al") 0xFEu8   // Значение 0xFE для команды перезагрузки
        );
    }
}

fn shutdown_action(buffer: &mut [[u8; COLS]; ROWS], row: usize) {
    let msg = b"Shutting down...";
    for (i, &byte) in msg.iter().enumerate() {
        write_char(row + 1, i, byte, 0x07); // Печатает на строке row + 1
        buffer[row + 1][i] = byte; // Записываем в буфер
    }

    unsafe {
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

pub fn command_fn(buffer: &mut [[u8; COLS]; ROWS], row: usize) {
    let command: &[u8] = &buffer[row][3..];

    let mut b = 0;
    for i in command.iter() {
        if *i != ('\0' as u8) {
            b += 1;
        }
    }

    b += 3;
    let comm: &[u8] = &buffer[row][3..b];

    let commands: [Command; 4] = [
        Command::new("hello", hello_action),
        Command::new("error", error_action),
        Command::new("reboot", reboot_action),
        Command::new("shutdown", shutdown_action),
    ];

    let mut found = false;
    for cmd in commands.iter() {
        if comm == cmd.name.as_bytes() {
            (cmd.action)(buffer, row);
            found = true;
            break;
        }
    }

    if !found {
        error_action(buffer, row);
    }
}
