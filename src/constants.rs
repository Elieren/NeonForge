pub const ROWS: usize = 25;
pub const COLS: usize = 80;
pub const MSG: &[u8; 3] = b"$: ";
pub const MAX_LINES: usize = 10; // Максимальное количество строк в логотипе

pub static mut CURRENT_ROW: usize = 0;
pub static mut CURRENT_COL: usize = 0;

pub const GPIO_BASE: usize = 0xFE200000; // Адрес для Raspberry Pi 4. Для других моделей может отличаться.
