let count = 0;

loop {
    tap("a");
    wait(1.0);
    tap("start");
    wait(1.0);

    count += 1;

    if count % 30 == 0 {
        screenshot(`smoke_${count}`);
    }
}
