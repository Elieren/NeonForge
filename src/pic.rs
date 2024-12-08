use x86_64::instructions::port::Port;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub struct ChainedPics {
    master_command: Port<u8>,
    master_data: Port<u8>,
    slave_command: Port<u8>,
    slave_data: Port<u8>,
}

impl ChainedPics {
    pub const unsafe fn new(_offset1: u8, _offset2: u8) -> Self {
        ChainedPics {
            master_command: Port::new(0x20),
            master_data: Port::new(0x21),
            slave_command: Port::new(0xA0),
            slave_data: Port::new(0xA1),
        }
    }

    pub unsafe fn initialize(&mut self) {
        let icw1 = 0x11; // Начальная команда
        let icw4 = 0x01; // 8086/88 (MCS-80/85) mode

        self.master_command.write(icw1);
        self.slave_command.write(icw1);

        self.master_data.write(PIC_1_OFFSET);
        self.slave_data.write(PIC_2_OFFSET);

        self.master_data.write(4); // Указывает на подключение Slave PIC на IRQ2
        self.slave_data.write(2); // Указывает на линию Slave PIC

        self.master_data.write(icw4);
        self.slave_data.write(icw4);
    }

    pub unsafe fn notify_end_of_interrupt(&mut self, irq: u8) {
        if irq >= 8 {
            self.slave_command.write(0x20);
        }
        self.master_command.write(0x20);
    }
}
