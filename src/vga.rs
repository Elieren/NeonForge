use crate::constants::{COLS, ROWS};

pub fn write_char(row: usize, col: usize, character: u8, color: u8) {
    let vga_buffer = 0xb8000 as *mut u8; // Адрес VGA буфера

    // Рассчитываем смещение, используя строки и столбцы
    let offset = (row * 80 + col) * 2;

    // Записываем символ в VGA буфер
    unsafe {
        *vga_buffer.offset(offset as isize) = character;
        *vga_buffer.offset(offset as isize + 1) = color;
    }
}

pub fn clear_screen(width: u16, height: u16) {
    let vga_buffer = 0xb8000 as *mut u8;
    for i in 0..(width as usize * height as usize * 2) {
        unsafe {
            *vga_buffer.offset(i as isize) = 0;
        }
    }
}

pub fn print_buffer(buffer: *mut [[u8; COLS]; ROWS]) {
    let width = COLS;
    unsafe {
        for row in 0..ROWS {
            for col in 0..COLS {
                if (*buffer)[row][col] != 0 {
                    let vga_buffer = 0xb8000 as *mut u8;
                    *vga_buffer.offset((row as isize * width as isize + col as isize) * 2) =
                        (*buffer)[row][col];
                    *vga_buffer.offset((row as isize * width as isize + col as isize) * 2 + 1) =
                        0x07;
                }
            }
        }
    }
}
