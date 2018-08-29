use super::*;
use generic_array::ArrayLength;
use std::ops::Deref;

pub struct Secure<N: ArrayLength<u8>> {
    ptr: NonNull<u8>,
    phantom: PhantomData<GenericArray<u8, N>>,
}

pub struct SecureGetRef<'a, N: ArrayLength<u8> + 'a> {
    secure: &'a Secure<N>,
}

const COUNT_SIZE: usize = mem::size_of::<AtomicUsize>();

impl<N: ArrayLength<u8>> Secure<N> {
    pub fn new<I>(init: I) -> Secure<N>
    where
        I: FnOnce(&mut GenericArray<u8, N>),
    {
        let len = ((N::to_usize() + 3 * COUNT_SIZE - 1) / COUNT_SIZE) * COUNT_SIZE;

        let ptr = unsafe {
            libsodium_sys::sodium_malloc(len) as *mut u8
        };
        unsafe {
            ptr::write_bytes(ptr, 0, len);
            let data_ptr = ptr.offset(2 * COUNT_SIZE as isize);
            let data_ptr = data_ptr as *mut GenericArray<u8, N>;
            init(&mut *data_ptr);
            libsodium_sys::sodium_mprotect_noaccess(ptr as *mut _);
        }
        let ptr = unwrap!(NonNull::new(ptr), "out of memory");
        let ret = Secure {
            ptr,
            phantom: PhantomData,
        };
        ret.inc_ref_count();
        ret
    }

    pub fn get_ref<'a>(&'a self) -> SecureGetRef<'a, N> {
        self.inc_reader_count();
        SecureGetRef {
            secure: self,
        }
    }

    fn ref_count<'a>(&'a self) -> &'a AtomicUsize {
        let ref_count_ptr = self.ptr.as_ptr() as *const u8 as *const AtomicUsize;
        unsafe {
            &*ref_count_ptr
        }
    }

    fn reader_count<'a>(&'a self) -> &'a AtomicUsize {
        unsafe {
            let reader_count_ptr = self.ptr.as_ptr().offset(COUNT_SIZE as isize);
            let reader_count_ptr = reader_count_ptr as *const u8 as *const AtomicUsize;
            &*reader_count_ptr
        }
    }

    fn data_ptr(&self) -> *const GenericArray<u8, N> {
        unsafe {
            let ptr = self.ptr.as_ptr().offset(2 * COUNT_SIZE as isize);
            ptr as *const u8 as *const GenericArray<u8, N>
        }
    }

    fn inc_ref_count(&self) {
        self.ref_count().fetch_add(1, Ordering::Relaxed);
    }

    fn dec_ref_count(&self) {
        if self.ref_count().fetch_sub(1, Ordering::Release) != 1 {
            return;
        }

        atomic::fence(Ordering::Acquire);
        unsafe {
            libsodium_sys::sodium_free(self.ptr.as_ptr() as *mut _);
        }
    }

    fn inc_reader_count(&self) {
        loop {
            let old_count = self.reader_count().swap(usize::MAX, Ordering::Acquire);
            if old_count == usize::MAX {
                thread::yield_now();
                continue;
            }

            if old_count == 0 {
                unsafe {
                    libsodium_sys::sodium_mprotect_readonly(self.ptr.as_ptr() as *mut _);
                }
            }

            let new_old_count = self.reader_count().swap(old_count + 1, Ordering::Release);
            debug_assert!(new_old_count == usize::MAX);
            break;
        }
    }

    fn dec_reader_count(&self) {
        loop {
            let old_count = self.reader_count().swap(usize::MAX, Ordering::Acquire);
            if old_count == usize::MAX {
                thread::yield_now();
                continue;
            }

            if old_count == 1 {
                unsafe {
                    libsodium_sys::sodium_mprotect_noaccess(self.ptr.as_ptr() as *mut _);
                }
            }

            let new_old_count = self.reader_count().swap(old_count - 1, Ordering::Release);
            debug_assert!(new_old_count == usize::MAX);
            break;
        }
    }
}

impl<N: ArrayLength<u8>> Drop for Secure<N> {
    fn drop(&mut self) {
        self.dec_ref_count()
    }
}

impl<N: ArrayLength<u8>> Clone for Secure<N> {
    fn clone(&self) -> Secure<N> {
        self.inc_ref_count();
        Secure {
            ptr: self.ptr,
            phantom: PhantomData,
        }
    }
}

impl<N: ArrayLength<u8>> PartialEq for Secure<N> {
    fn eq(&self, other: &Secure<N>) -> bool {
        0 == unsafe {
            libsodium_sys::sodium_memcmp(
                self.data_ptr() as *const _,
                other.data_ptr() as *const _,
                N::to_usize(),
            )
        }
    }
}

impl<N: ArrayLength<u8>> Eq for Secure<N> {}

impl<'a, N: ArrayLength<u8>> Drop for SecureGetRef<'a, N> {
    fn drop(&mut self) {
        self.secure.dec_reader_count();
    }
}

impl<'a, N: ArrayLength<u8>> Deref for SecureGetRef<'a, N> {
    type Target = GenericArray<u8, N>;

    fn deref(&self) -> &GenericArray<u8, N> {
        let ptr = self.secure.data_ptr();
        unsafe {
            &*ptr
        }
    }
}

