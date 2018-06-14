pub trait VecExt {
    fn zeros(len: usize) -> Vec<u8>;
}

impl VecExt for Vec<u8> {
    fn zeros(len: usize) -> Vec<u8> {
        let mut ret = Vec::with_capacity(len);
        unsafe {
            ret.set_len(len);
            for x in &mut ret {
                *x = 0;
            }
        }
        ret
    }
}

