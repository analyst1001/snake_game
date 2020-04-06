use x86_64::instructions::port::Port;

const CMOS_ADDRESS_PORT: u16 = 0x70;
const CMOS_DATA_PORT : u16 = 0x71;
const STATUS_REGISTER_A: u8 = 0x0A;
const STATUS_REGISTER_B: u8 = 0x0B;
const SECOND_REGISTER: u8 = 0x00;
const MINUTE_REGISTER: u8 = 0x02;
const HOUR_REGISTER: u8 = 0x04;
const DAY_REGISTER: u8 = 0x07;
const MONTH_REGISTER: u8 = 0x08;
const YEAR_REGISTER: u8 = 0x09;

fn get_update_in_progress_flag(cmos_address_port: &mut Port<u8>, cmos_data_port: &mut Port<u8>) -> u8 {
    unsafe { cmos_address_port.write(STATUS_REGISTER_A) };
    let status = unsafe { cmos_data_port.read() & 0x80 };
    status
}

fn get_rtc_register(cmos_address_port: &mut Port<u8>, cmos_data_port: &mut Port<u8>, reg: u8) -> u8 {
    unsafe { cmos_address_port.write(reg) };
    let value = unsafe { cmos_data_port.read() };
    value
}


/// Get some seed from system time. We do not need exact system time, so
/// this is not a proper implementation to get system time
/// Based upon https://wiki.osdev.org/CMOS#Reading_All_RTC_Time_and_Date_Registers
pub fn get_system_time_seed() -> u64 {
    let mut cmos_address_port = Port::new(CMOS_ADDRESS_PORT);
    let mut cmos_data_port = Port::new(CMOS_DATA_PORT);
    let mut second: u64 = 0;
    let mut minute: u64 = 0;
    let mut hour: u64 = 0;
    let mut day: u64 = 0;
    let mut month: u64 = 0;
    //let mut year: u8 = 0;
    while (get_update_in_progress_flag(&mut cmos_address_port, &mut cmos_data_port) != 0) {
        second = get_rtc_register(&mut cmos_address_port, &mut cmos_data_port, SECOND_REGISTER) as u64;
        minute = get_rtc_register(&mut cmos_address_port, &mut cmos_data_port, MINUTE_REGISTER) as u64;
        hour = get_rtc_register(&mut cmos_address_port, &mut cmos_data_port, HOUR_REGISTER) as u64;
        day = get_rtc_register(&mut cmos_address_port, &mut cmos_data_port, DAY_REGISTER) as u64;
        month = get_rtc_register(&mut cmos_address_port, &mut cmos_data_port, MONTH_REGISTER) as u64;
        //year = get_rtc_register(&mut cmos_address_port, &mut cmos_data_port, YEAR_REGISTER) as u64;
    }
    let register_b_status = get_rtc_register(&mut cmos_address_port, &mut cmos_data_port, STATUS_REGISTER_B);
    if ((register_b_status & 0x04) == 0) {
       second = (second & 0x0F) + ((second / 16) * 10);
       minute = (minute & 0x0F) + ((minute / 16) * 10);
       hour = ((hour & 0x0F) +  (((hour & 0x70) / 16) * 10) ) | (hour & 0x80);
       day = (day & 0x0F) + ((day / 16) * 10);
       month = (month & 0x0F) + ((month / 16) * 10);
       //year = (year & 0x0F) + ((year / 16) * 10);
    }

    if ((register_b_status & 0x02) == 0 && (hour & 0x80) != 0) {
        hour = ((hour & 0x7F) + 12) % 24;
    }
    // Naive usage of 30 days month
    return (second + (minute * 60) + (hour * 3600) + (day * 86400) + (month * 2592000));
}
