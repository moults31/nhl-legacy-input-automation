use thiserror::Error;

#[derive(Error, Debug)]
pub enum ControllerError {
    #[error("device creation failed: {0}")]
    DeviceCreation(String),
    #[error("input error: {0}")]
    InputError(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Button {
    A,
    B,
    X,
    Y,
    Start,
    Back,
    LeftBumper,
    RightBumper,
    LeftThumb,
    RightThumb,
    Guide,
    DpadNorth,
    DpadSouth,
    DpadEast,
    DpadWest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Axis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
    LeftTrigger,
    RightTrigger,
    DpadX,
    DpadY,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stick {
    Left,
    Right,
}

pub trait Controller: Send {
    fn press(&mut self, button: Button) -> Result<(), ControllerError>;
    fn release(&mut self, button: Button) -> Result<(), ControllerError>;
    fn set_axis(&mut self, axis: Axis, value: f64) -> Result<(), ControllerError>;
    fn set_stick(&mut self, stick: Stick, x: f64, y: f64) -> Result<(), ControllerError>;
    fn flush(&mut self) -> Result<(), ControllerError>;
}
