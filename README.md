# scene-scheduler

A handy application to plan scenes for your theater piece.

## Dependencies

### Rust

Tested on `rustc >= 1.69.0`.

### Additional Libraries

- `libfontconfig-dev`. Ubuntu: `sudo apt install libfontconfig-dev`.

## Dev Plan

- [x] Read in excel file of schedule and shift plan
- [x] Sort schedule based on shift plan
- [x] Display it in nice GUI
- [x] Export to `.ical` file
- [ ] Extend to other file formats
