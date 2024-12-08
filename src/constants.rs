pub const ROWS: usize = 25;
pub const COLS: usize = 80;
pub const MSG: &[u8; 3] = b"$: ";
pub const MAX_LINES: usize = 10; // Максимальное количество строк в логотипе

pub static mut CURRENT_ROW: usize = 0;
pub static mut CURRENT_COL: usize = 0;

pub const GPIO_BASE: usize = 0xFE200000; // Адрес для Raspberry Pi 4. Для других моделей может отличаться.

pub const COLOR_STATUS_BAR: u8 = 0xee;
pub const COLOR_INFO: u8 = 0xe0;

pub const HEAP_SIZE: usize = 1024 * 1024; // 1 MiB
pub const PARTITION_OFFSET: usize = 1048576; // 1 MiБ
