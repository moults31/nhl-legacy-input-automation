# Setup

## uinput permissions

The virtual controller uses the Linux `uinput` kernel interface to create a
virtual device at `/dev/uinput`.

### Option 1: udev rule (recommended)

```sh
echo 'KERNEL=="uinput", SUBSYSTEM=="misc", MODE="0660", GROUP="input"' \
  | sudo tee /etc/udev/rules.d/99-uinput.rules
sudo udevadm control --reload-rules
sudo udevadm trigger
sudo usermod -aG input "$USER"
```

Log out and back in for the group change to take effect.

### Option 2: run as root

```sh
sudo cargo run -- --script scripts/examples/spam-a-start.rhai
```

(Not recommended; udev is cleaner.)

## Verify the virtual controller

After running `nhl-input`, open another terminal:

```sh
evtest /dev/input/event*
```

Look for the device named "Microsoft X-Box One pad".
