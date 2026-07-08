use evdev::uinput::VirtualDevice;
use evdev::{AbsoluteAxisCode, AttributeSet, KeyCode, SynchronizationCode};
use tracing::debug;

use crate::traits::{Axis, Button, Controller, ControllerError, Stick};
use crate::xbox_profile;

const EV_KEY: u16 = 0x01;
const EV_ABS: u16 = 0x03;
const EV_SYN: u16 = 0x00;

pub struct UinputController {
    device: evdev::uinput::VirtualDevice,
}

impl UinputController {
    pub fn new() -> Result<Self, ControllerError> {
        let keys = AttributeSet::from_iter([
            KeyCode::BTN_SOUTH,
            KeyCode::BTN_EAST,
            KeyCode::BTN_NORTH,
            KeyCode::BTN_WEST,
            KeyCode::BTN_START,
            KeyCode::BTN_SELECT,
            KeyCode::BTN_TL,
            KeyCode::BTN_TR,
            KeyCode::BTN_THUMBL,
            KeyCode::BTN_THUMBR,
            KeyCode::BTN_MODE,
        ]);

        let device = VirtualDevice::builder()
            .map_err(|e| ControllerError::DeviceCreation(e.to_string()))?
            .name(xbox_profile::DEVICE_NAME)
            .input_id(evdev::InputId::new(
                evdev::BusType::BUS_USB,
                xbox_profile::VID,
                xbox_profile::PID,
                xbox_profile::VERSION,
            ))
            .with_keys(&keys)
            .map_err(|e| ControllerError::DeviceCreation(e.to_string()))?
            .with_absolute_axis(&evdev::UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_X,
                evdev::AbsInfo::new(
                    0,
                    xbox_profile::STICK_MIN,
                    xbox_profile::STICK_MAX,
                    xbox_profile::STICK_FUZZ,
                    xbox_profile::STICK_FLAT,
                    0,
                ),
            ))
            .map_err(|e| ControllerError::DeviceCreation(e.to_string()))?
            .with_absolute_axis(&evdev::UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_Y,
                evdev::AbsInfo::new(
                    0,
                    xbox_profile::STICK_MIN,
                    xbox_profile::STICK_MAX,
                    xbox_profile::STICK_FUZZ,
                    xbox_profile::STICK_FLAT,
                    0,
                ),
            ))
            .map_err(|e| ControllerError::DeviceCreation(e.to_string()))?
            .with_absolute_axis(&evdev::UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_RX,
                evdev::AbsInfo::new(
                    0,
                    xbox_profile::STICK_MIN,
                    xbox_profile::STICK_MAX,
                    xbox_profile::STICK_FUZZ,
                    xbox_profile::STICK_FLAT,
                    0,
                ),
            ))
            .map_err(|e| ControllerError::DeviceCreation(e.to_string()))?
            .with_absolute_axis(&evdev::UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_RY,
                evdev::AbsInfo::new(
                    0,
                    xbox_profile::STICK_MIN,
                    xbox_profile::STICK_MAX,
                    xbox_profile::STICK_FUZZ,
                    xbox_profile::STICK_FLAT,
                    0,
                ),
            ))
            .map_err(|e| ControllerError::DeviceCreation(e.to_string()))?
            .with_absolute_axis(&evdev::UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_Z,
                evdev::AbsInfo::new(
                    0,
                    xbox_profile::TRIGGER_MIN,
                    xbox_profile::TRIGGER_MAX,
                    0,
                    0,
                    0,
                ),
            ))
            .map_err(|e| ControllerError::DeviceCreation(e.to_string()))?
            .with_absolute_axis(&evdev::UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_RZ,
                evdev::AbsInfo::new(
                    0,
                    xbox_profile::TRIGGER_MIN,
                    xbox_profile::TRIGGER_MAX,
                    0,
                    0,
                    0,
                ),
            ))
            .map_err(|e| ControllerError::DeviceCreation(e.to_string()))?
            .with_absolute_axis(&evdev::UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_HAT0X,
                evdev::AbsInfo::new(0, xbox_profile::HAT_MIN, xbox_profile::HAT_MAX, 0, 0, 0),
            ))
            .map_err(|e| ControllerError::DeviceCreation(e.to_string()))?
            .with_absolute_axis(&evdev::UinputAbsSetup::new(
                AbsoluteAxisCode::ABS_HAT0Y,
                evdev::AbsInfo::new(0, xbox_profile::HAT_MIN, xbox_profile::HAT_MAX, 0, 0, 0),
            ))
            .map_err(|e| ControllerError::DeviceCreation(e.to_string()))?
            .build()
            .map_err(|e| ControllerError::DeviceCreation(e.to_string()))?;

        debug!("uinput device created: {}", xbox_profile::DEVICE_NAME);

        Ok(Self { device })
    }

    fn key_for(button: Button) -> KeyCode {
        match button {
            Button::A => KeyCode::BTN_SOUTH,
            Button::B => KeyCode::BTN_EAST,
            Button::X => KeyCode::BTN_WEST,
            Button::Y => KeyCode::BTN_NORTH,
            Button::Start => KeyCode::BTN_START,
            Button::Back => KeyCode::BTN_SELECT,
            Button::LeftBumper => KeyCode::BTN_TL,
            Button::RightBumper => KeyCode::BTN_TR,
            Button::LeftThumb => KeyCode::BTN_THUMBL,
            Button::RightThumb => KeyCode::BTN_THUMBR,
            Button::Guide => KeyCode::BTN_MODE,
            _ => panic!("non-key button {button:?} used as key"),
        }
    }
}

impl Controller for UinputController {
    fn press(&mut self, button: Button) -> Result<(), ControllerError> {
        if matches!(
            button,
            Button::DpadNorth | Button::DpadSouth | Button::DpadEast | Button::DpadWest
        ) {
            self.set_axis(
                dpad_button_to_axis(button),
                dpad_button_to_value(button, true),
            )?;
        } else {
            self.device
                .emit(&[evdev::InputEvent::new(EV_KEY, Self::key_for(button).0, 1)])
                .map_err(|e| ControllerError::InputError(e.to_string()))?;
        }
        Ok(())
    }

    fn release(&mut self, button: Button) -> Result<(), ControllerError> {
        if matches!(
            button,
            Button::DpadNorth | Button::DpadSouth | Button::DpadEast | Button::DpadWest
        ) {
            self.set_axis(dpad_button_to_axis(button), 0.0)?;
        } else {
            self.device
                .emit(&[evdev::InputEvent::new(EV_KEY, Self::key_for(button).0, 0)])
                .map_err(|e| ControllerError::InputError(e.to_string()))?;
        }
        Ok(())
    }

    fn set_axis(&mut self, axis: Axis, value: f64) -> Result<(), ControllerError> {
        let (abs_type, raw) = match axis {
            Axis::LeftStickX => (AbsoluteAxisCode::ABS_X, stick_to_raw(value)),
            Axis::LeftStickY => (AbsoluteAxisCode::ABS_Y, stick_to_raw(-value)),
            Axis::RightStickX => (AbsoluteAxisCode::ABS_RX, stick_to_raw(value)),
            Axis::RightStickY => (AbsoluteAxisCode::ABS_RY, stick_to_raw(-value)),
            Axis::LeftTrigger => (AbsoluteAxisCode::ABS_Z, trigger_to_raw(value)),
            Axis::RightTrigger => (AbsoluteAxisCode::ABS_RZ, trigger_to_raw(value)),
            Axis::DpadX => (AbsoluteAxisCode::ABS_HAT0X, hat_to_raw(value)),
            Axis::DpadY => (AbsoluteAxisCode::ABS_HAT0Y, hat_to_raw(-value)),
        };
        self.device
            .emit(&[evdev::InputEvent::new(EV_ABS, abs_type.0, raw)])
            .map_err(|e| ControllerError::InputError(e.to_string()))?;
        Ok(())
    }

    fn set_stick(&mut self, stick: Stick, x: f64, y: f64) -> Result<(), ControllerError> {
        let (axis_x, axis_y) = match stick {
            Stick::Left => (Axis::LeftStickX, Axis::LeftStickY),
            Stick::Right => (Axis::RightStickX, Axis::RightStickY),
        };
        self.set_axis(axis_x, x)?;
        self.set_axis(axis_y, y)?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), ControllerError> {
        self.device
            .emit(&[evdev::InputEvent::new(
                EV_SYN,
                SynchronizationCode::SYN_REPORT.0,
                0,
            )])
            .map_err(|e| ControllerError::InputError(e.to_string()))?;
        Ok(())
    }
}

fn stick_to_raw(value: f64) -> i32 {
    let clamped = value.clamp(-1.0, 1.0);
    (clamped * f64::from(xbox_profile::STICK_MAX)) as i32
}

fn trigger_to_raw(value: f64) -> i32 {
    let clamped = value.clamp(0.0, 1.0);
    (clamped * f64::from(xbox_profile::TRIGGER_MAX)) as i32
}

fn hat_to_raw(value: f64) -> i32 {
    let clamped = value.clamp(-1.0, 1.0);
    clamped as i32
}

fn dpad_button_to_axis(button: Button) -> Axis {
    match button {
        Button::DpadEast | Button::DpadWest => Axis::DpadX,
        Button::DpadNorth | Button::DpadSouth => Axis::DpadY,
        _ => unreachable!(),
    }
}

fn dpad_button_to_value(button: Button, pressed: bool) -> f64 {
    if !pressed {
        return 0.0;
    }
    match button {
        Button::DpadEast => 1.0,
        Button::DpadWest => -1.0,
        Button::DpadNorth => 1.0,
        Button::DpadSouth => -1.0,
        _ => unreachable!(),
    }
}
