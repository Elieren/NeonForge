use core::sync::atomic::{AtomicUsize, Ordering};
use spin::Mutex;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

static TICKS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Copy)]
pub struct Time {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
}

static CURRENT_TIME: Mutex<Time> = Mutex::new(Time {
    hours: 0,
    minutes: 0,
    seconds: 0,
});

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();
static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard = PIC_1_OFFSET + 1,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

extern "x86-interrupt" fn pit_interrupt_handler(_stack_frame: InterruptStackFrame) {
    TICKS.fetch_add(1, Ordering::Relaxed);

    if TICKS.load(Ordering::Relaxed) % 1000 == 0 {
        let mut time = CURRENT_TIME.lock();
        time.seconds += 1;

        if time.seconds >= 60 {
            time.seconds = 0;
            time.minutes += 1;
            if time.minutes >= 60 {
                time.minutes = 0;
                time.hours += 1;
                if time.hours >= 24 {
                    time.hours = 0;
                }
            }
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        let mut port = Port::new(0x60);
        let _scancode: u8 = port.read();

        // Здесь можно добавить обработку кода клавиши

        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

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

pub fn init_idt() {
    unsafe {
        IDT[InterruptIndex::Timer.as_usize()].set_handler_fn(pit_interrupt_handler);
        IDT[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        let idt = &raw mut IDT;
        idt.as_ref().expect("IDT is None").load();
        PICS.lock().initialize();
    }
}

pub fn enable_interrupts() {
    x86_64::instructions::interrupts::enable();
}

pub fn get_time() -> Time {
    let time = CURRENT_TIME.lock();
    *time
}

pub fn set_time(hours: u8, minutes: u8, seconds: u8) {
    let mut time = CURRENT_TIME.lock();
    time.hours = hours;
    time.minutes = minutes;
    time.seconds = seconds;
}

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
