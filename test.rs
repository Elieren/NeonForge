use crate::constants::{COLS, ROWS};
use crate::delay;
use crate::eng::SCANCODE_MAP;
use crate::vga::write_char;

struct SimpleString {
    buffer: [u8; 128],
    length: usize,
}

impl SimpleString {
    fn new() -> Self {
        SimpleString {
            buffer: [0; 128],
            length: 0,
        }
    }

    fn push_str(&mut self, s: &str) {
        let bytes = s.as_bytes();
        let len = bytes.len();
        if self.length + len <= self.buffer.len() {
            self.buffer[self.length..self.length + len].copy_from_slice(bytes);
            self.length += len;
        }
    }

    fn push_char(&mut self, c: char) {
        let mut char_buffer = [0; 4];
        let char_str = c.encode_utf8(&mut char_buffer);
        self.push_str(char_str);
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buffer[..self.length]).unwrap()
    }

    fn len(&self) -> usize {
        self.length
    }
}

fn bytes_to_significant_str(buffer: &[[u8; COLS]; ROWS], row: usize, start: usize) -> SimpleString {
    let mut result = SimpleString::new();

    // Идем по строке буфера от начала до конца
    for &byte in &buffer[row][start..] {
        // Проверяем, что байт не ноль и что символ уникальный
        if byte != 0 {
            // Логика SCANCODE_MAP для преобразования байта в символ
            if let Some(character) = SCANCODE_MAP[byte as usize] {
                // Добавляем символ в результат
                result.push_char(character);

                // Лог добавления символа
                write_char(20, result.len(), character as u8, 0x07);
            }
        }
    }

    // Лог для длины строки после обработки
    write_char(20, 10, b'L', 0x07);
    write_char(20, 11, b'N', 0x07);
    write_char(20, 12, b':', 0x07);
    write_char(20, 13, b' ', 0x07);
    write_char(20, 14, b'0' + (result.len() / 10) as u8, 0x07);
    write_char(20, 15, b'0' + (result.len() % 10) as u8, 0x07);

    result
}

pub fn command_fn(buffer: &mut [[u8; COLS]; ROWS], row: usize) {
    let command_str = bytes_to_significant_str(buffer, row, 3);

    // Выводим строку команды для проверки
    write_char(row + 2, 0, b'L', 0x07);
    for (i, &byte) in command_str.as_str().as_bytes().iter().enumerate() {
        write_char(row + 2, i + 1, byte, 0x07);
    }
    write_char(row + 3, 0, b'E', 0x07);
    delay(20000000);

    // Выводим длину строки команды
    let cmd_len = command_str.len();
    write_char(row + 6, 0, b'L', 0x07);
    write_char(row + 6, 1, b'N', 0x07);
    write_char(row + 6, 2, b':', 0x07);
    write_char(row + 6, 3, b' ', 0x07);
    write_char(row + 6, 4, b'0' + (cmd_len / 10) as u8, 0x07);
    write_char(row + 6, 5, b'0' + (cmd_len % 10) as u8, 0x07);
    delay(20000000);

    // Лог состояния буфера
    for (i, &byte) in buffer[row].iter().enumerate() {
        write_char(21, i * 2, byte, 0x07);
        if byte == 0 {
            write_char(21, i * 2 + 1, b'0', 0x07);
        } else {
            write_char(21, i * 2 + 1, byte, 0x07);
        }
    }
    delay(20000000);

    if command_str.as_str() == "hello" {
        write_char(row + 3, 2, b'X', 0x07); // Лог успешного сравнения
        let msg = b"Hello!";
        for (i, &byte) in msg.iter().enumerate() {
            write_char(row + 4, i, byte, 0x07);
            buffer[row + 4][i] = byte;
            delay(100000);
        }
    } else {
        // Лог неуспешного сравнения
        write_char(row + 3, 2, b'F', 0x07);
        for (i, &byte) in b"failed".iter().enumerate() {
            write_char(row + 4, i, byte, 0x07);
        }
    }
}
