use time::UtcOffset;

#[link_section = ".rtc.data"]
pub static mut UTC_OFFSET: Option<UtcOffset> = None;

#[link_section = ".rtc.data"]
pub static mut RTC_INITIALIZED: bool = false;
