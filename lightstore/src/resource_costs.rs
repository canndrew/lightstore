use super::*;

// This function is supposed to give us a measure of the cost of RAM usage. This is used when
// when deciding how to optimize/balance certain parts of the implementation.
//
// Ideally we should have some way of measuring the system RAM usage and the daemon's RAM usage.
// This, combined with some user configuration options should determine how willing we are to
// consume more RAM.
pub fn memory() -> BtcPerByteSec {
    // Fermi estimate of a good value picked by looking at RAM prices
    BtcPerByteSec(3e-19)
}

// This function is supposed to give us a measure of the cost of upload bandwidth. This should take
// into account the user's internet expenses, the total bandwidth available, as well as other
// consumers of the system's bandwidth which may be important to the user.
pub fn upload() -> BtcPerByte {
    // Fermi estimate of a good value picked by looking at local internet plan prices
    BtcPerByte(1e-13)
}

pub fn download() -> BtcPerByte {
    // Fermi estimate of a good value picked by looking at local internet plan prices
    BtcPerByte(1e-13)
}

