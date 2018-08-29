use std::time::Duration;

use std::ops::{
    Neg, Add, Sub, Rem, Mul, Div,
    AddAssign, SubAssign, RemAssign, MulAssign, DivAssign,
};

macro_rules! dim {
    ($name:ident) => {
        #[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
        pub struct $name(pub f64);

        impl $name {
            pub fn val(self) -> f64 {
                self.0
            }
        }

        impl Neg for $name {
            type Output = $name;

            fn neg(self) -> $name {
                $name(-self.val())
            }
        }

        impl Add<$name> for $name {
            type Output = $name;

            fn add(self, other: $name) -> $name {
                $name(self.val() + other.val())
            }
        }

        impl AddAssign<$name> for $name {
            fn add_assign(&mut self, other: $name) {
                self.0 += other.val();
            }
        }

        impl Sub<$name> for $name {
            type Output = $name;

            fn sub(self, other: $name) -> $name {
                $name(self.val() - other.val())
            }
        }

        impl SubAssign<$name> for $name {
            fn sub_assign(&mut self, other: $name) {
                self.0 -= other.val();
            }
        }

        impl Rem<$name> for $name {
            type Output = $name;

            fn rem(self, other: $name) -> $name {
                $name(self.val() % other.val())
            }
        }

        impl RemAssign<$name> for $name {
            fn rem_assign(&mut self, other: $name) {
                self.0 %= other.val();
            }
        }

        impl Mul<f64> for $name {
            type Output = $name;

            fn mul(self, other: f64) -> $name {
                $name(self.val() * other)
            }
        }

        impl MulAssign<f64> for $name {
            fn mul_assign(&mut self, other: f64) {
                self.0 *= other;
            }
        }

        impl Div<f64> for $name {
            type Output = $name;

            fn div(self, other: f64) -> $name {
                $name(self.val() / other)
            }
        }

        impl DivAssign<f64> for $name {
            fn div_assign(&mut self, other: f64) {
                self.0 /= other;
            }
        }

        impl Div<$name> for $name {
            type Output = f64;

            fn div(self, other: $name) -> f64 {
                self.val() / other.val()
            }
        }
    }
}

macro_rules! square {
    ($name:ident ^ 2 -> $result:ident) => {
        impl Mul<$name> for $name {
            type Output = $result;

            fn mul(self, other: $name) -> $result {
                $result(self.val() * other.val())
            }
        }

        impl Div<$name> for $result {
            type Output = $name;

            fn div(self, other: $name) -> $name {
                $name(self.val() / other.val())
            }
        }
    }
}

macro_rules! mul {
    ($left:ident * $right:ident -> $result:ident) => {
        impl Mul<$right> for $left {
            type Output = $result;

            fn mul(self, other: $right) -> $result {
                $result(self.val() * other.val())
            }
        }

        impl Mul<$left> for $right {
            type Output = $result;

            fn mul(self, other: $left) -> $result {
                $result(self.val() * other.val())
            }
        }

        impl Div<$left> for $result {
            type Output = $right;

            fn div(self, other: $left) -> $right {
                $right(self.val() / other.val())
            }
        }

        impl Div<$right> for $result {
            type Output = $left;

            fn div(self, other: $right) -> $left {
                $left(self.val() / other.val())
            }
        }
    }
}

macro_rules! inv {
    ($dim:ident ^ -1 -> $inv:ident) => {
        impl Mul<$dim> for $inv {
            type Output = f64;

            fn mul(self, other: $dim) -> f64 {
                self.val() * other.val()
            }
        }

        impl Mul<$inv> for $dim {
            type Output = f64;

            fn mul(self, other: $inv) -> f64 {
                self.val() * other.val()
            }
        }

        impl Div<$dim> for f64 {
            type Output = $inv;

            fn div(self, other: $dim) -> $inv {
                $inv(self / other.val())
            }
        }

        impl Div<$inv> for f64 {
            type Output = $dim;

            fn div(self, other: $inv) -> $dim {
                $dim(self / other.val())
            }
        }
    }
}

macro_rules! log {
    (log($dim:ident) -> $name:ident) => {
        #[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
        pub struct $name(pub f64);

        impl $name {
            pub fn val(&self) -> f64 {
                self.0
            }

            pub fn exp(&self) -> $dim {
                $dim(self.val().exp())
            }
        }

        impl Add<f64> for $name {
            type Output = $name;

            fn add(self, arg: f64) -> $name {
                $name(self.val() + arg)
            }
        }

        impl Sub<f64> for $name {
            type Output = $name;

            fn sub(self, arg: f64) -> $name {
                $name(self.val() - arg)
            }
        }

        impl Sub<$name> for $name {
            type Output = f64;

            fn sub(self, arg: $name) -> f64 {
                self.val() - arg.val()
            }
        }

        impl $dim {
            pub fn log(&self) -> $name {
                $name(self.val().ln())
            }
        }
    }
}

macro_rules! double {
    ($half:ident * 2 -> $double:ident) => {
        impl Add<$half> for $half {
            type Output = $double;

            fn add(self, arg: $half) -> $double {
                $double(self.val() + arg.val())
            }
        }

        impl Sub<$half> for $double {
            type Output = $half;

            fn sub(self, arg: $half) -> $half {
                $half(self.val() - arg.val())
            }
        }

        impl $half {
            pub fn double(self) -> $double {
                $double(self.val() * 2.0)
            }
        }

        impl $double {
            pub fn half(self) -> $half {
                $half(self.val() / 2.0)
            }
        }
    }
}

dim!(Btc);
dim!(Btc2);
square!(Btc ^ 2 -> Btc2);
log!(log(Btc) -> LogBtc);
log!(log(Btc2) -> LogBtc2);
double!(LogBtc * 2 -> LogBtc2);

dim!(Byte);

dim!(Sec);
dim!(PerSec);
inv!(Sec ^ -1 -> PerSec);

dim!(BtcPerByte);
dim!(Btc2PerByte2);
square!(BtcPerByte ^ 2 -> Btc2PerByte2);
log!(log(BtcPerByte) -> LogBtcPerByte);
log!(log(Btc2PerByte2) -> LogBtc2PerByte2);
double!(LogBtcPerByte * 2 -> LogBtc2PerByte2);
mul!(BtcPerByte * Byte -> Btc);

dim!(BtcPerSec);
mul!(Btc * PerSec -> BtcPerSec);

dim!(ByteSec);
dim!(SecPerByte);
mul!(Byte * Sec -> ByteSec);
mul!(SecPerByte * Byte -> Sec);

dim!(BtcPerByteSec);
mul!(BtcPerByteSec * ByteSec -> Btc);

impl From<Duration> for Sec {
    fn from(duration: Duration) -> Sec {
        Sec((duration.as_secs() as f64) + (1e-9 * duration.subsec_nanos() as f64))
    }
}

impl From<usize> for Byte {
    fn from(size: usize) -> Byte {
        Byte(size as f64)
    }
}

