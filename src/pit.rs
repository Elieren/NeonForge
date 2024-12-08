use x86_64::instructions::port::Port;

pub fn init_pit() {
    let frequency: u16 = 1193; // Частота таймера ~1мс (1193182 / 1000)

    unsafe {
        let mut command_port = Port::new(0x43);
        command_port.write(0x34 as u8); // Управление PIT: канал 0, режим 2

        let mut data_port = Port::new(0x40);
        data_port.write((frequency & 0xFF) as u8); // Младший байт
        data_port.write((frequency >> 8) as u8); // Старший байт
    }
}
