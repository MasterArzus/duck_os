//! 这个模块是用来在多线程中保证数据一致性。
//！而无需使用锁之类的同步语句。

/// 
pub struct SyncUnsafeCell<T>(core::cell::SyncUnsafeCell<T>);

impl <T> SyncUnsafeCell<T> {
    ///
    #[inline]
    pub fn new(value: T) -> Self {
        Self(core::cell::SyncUnsafeCell::new(value))
    }

    /// TODO: 什么时候使用unchecked_mut 或者是 get_mut 仍然是一个问题
    /// TODO: 现在先不管它，全部使用 get_unchecked_mut 函数
    pub fn get_unchecked_mut(&self) -> &mut T {
        unsafe {
            &mut *self.0.get()
        }
    }

    ///
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.0.get_mut()
    }

    ///
    #[inline]
    pub fn get(&self) -> *mut T {
        self.0.get()
    }
}