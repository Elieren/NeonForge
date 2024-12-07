use crate::constants::GPIO_BASE;
use core::arch::asm;

#[repr(C)]
pub struct Gpio {
    fsel: [u32; 6],
    set: [u32; 2],
    clr: [u32; 2],
    lev: [u32; 2],
}

impl Gpio {
    pub fn new() -> &'static mut Gpio {
        unsafe { &mut *(GPIO_BASE as *mut Gpio) }
    }

    pub fn set_mode(&mut self, pin: usize, mode: u32) {
        let reg = pin / 10;
        let shift = (pin % 10) * 3;
        self.fsel[reg] = (self.fsel[reg] & !(0b111 << shift)) | ((mode & 0b111) << shift);
    }

    pub fn write(&mut self, pin: usize, value: bool) {
        if value {
            self.set[pin / 32] = 1 << (pin % 32);
        } else {
            self.clr[pin / 32] = 1 << (pin % 32);
        }
    }

    pub fn read(&self, pin: usize) -> bool {
        (self.lev[pin / 32] & (1 << (pin % 32))) != 0
    }
}

pub fn is_raspberry_pi_4() -> bool {
    // Проверка регистра MIDR_EL1 (Main ID Register) для определения модели процессора ARM
    let midr_el1: u64;
    unsafe {
        asm!("mrs {}, MIDR_EL1", out(reg) midr_el1);
    }

    // Значение регистра для конкретных моделей процессоров ARM
    const BROADCOM_ARM_CORTEX_A72: u64 = 0x410FD034;

    // Проверяем, соответствует ли значение регистру для ARM Cortex-A72 (используется в Raspberry Pi 4)
    midr_el1 == BROADCOM_ARM_CORTEX_A72
}
