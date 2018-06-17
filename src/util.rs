pub union MaybeUninit<T: Copy> {
    pub init: T,
    pub uninit: (),
}

