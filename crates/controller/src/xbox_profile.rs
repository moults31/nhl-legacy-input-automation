pub const VID: u16 = 0x045E;
pub const PID: u16 = 0x02FF;
pub const VERSION: u16 = 0x0111;
pub const DEVICE_NAME: &str = "Microsoft X-Box One pad";

pub const STICK_MIN: i32 = -32768;
pub const STICK_MAX: i32 = 32767;
pub const STICK_FUZZ: i32 = 16;
pub const STICK_FLAT: i32 = 128;

pub const TRIGGER_MIN: i32 = 0;
pub const TRIGGER_MAX: i32 = 1023;

pub const HAT_MIN: i32 = -1;
pub const HAT_MAX: i32 = 1;
