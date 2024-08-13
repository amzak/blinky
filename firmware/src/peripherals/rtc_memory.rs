use time::UtcOffset;

#[link_section = ".rtc.data"]
pub static mut UTC_OFFSET: Option<UtcOffset> = None;
