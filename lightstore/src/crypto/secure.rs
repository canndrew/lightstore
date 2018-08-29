use super::*;

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone)]
pub struct Secure<T: ZeroMem> {
    inner: Arc<SecureInner<T>>,
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash)]
struct SecureInner<T: ZeroMem> {
    val: T,
}

impl<T: ZeroMem> Secure<T> {
    pub fn move_from(val: &mut T) -> Secure<T> {
        let inner = Arc::new(SecureInner { val: T::ZERO_MEM });
        let ptr_out = &inner.val as *const T as *mut T;
        let ptr_in = val as *mut T;
        unsafe {
            ptr::copy_nonoverlapping(ptr_in, ptr_out, 1);
            zero_mem(ptr_in);
        }
        Secure { inner }
    }
}

impl<T: ZeroMem> Drop for SecureInner<T> {
    fn drop(&mut self) {
        let ptr_out = &self.val as *const T as *mut T;
        unsafe {
            zero_mem(ptr_out);
        }
    }
}

impl<T: ZeroMem> Deref for Secure<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner.val
    }
}

impl<T: ZeroMem + Hash> fmt::Debug for Secure<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut hasher = DefaultHasher::new();
        <T as Hash>::hash(self, &mut hasher);
        write!(fmt, "Secure({:016x})", hasher.finish())
    }
}

pub unsafe trait ZeroMem {
    const ZERO_MEM: Self;
}

unsafe impl ZeroMem for [u8; 32] {
    const ZERO_MEM: [u8; 32] = [0u8; 32];
}

unsafe fn zero_mem<T: ZeroMem>(val: *mut T) {
    // TODO: Use SecureZeroMemory on windows.
    // TODO: Use memset_s when it's available in libc
    let start = val as *mut u8;
    for i in 0..mem::size_of::<T>() {
        let p = start.offset(i as isize);
        ptr::write_volatile(p, 0u8);
    }
}

