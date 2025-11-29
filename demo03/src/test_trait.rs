use std::sync::{Mutex, PoisonError};

pub trait LockExt<T> {
    // 1. 闭包现在接受 &mut T (可变引用)
    fn with_lock<F, R>(&self, f: F) -> Result<R, PoisonError<R>>
    where
        F: FnOnce(&mut T) -> R; // <-- 修复: &mut T
}

impl<T> LockExt<T> for Mutex<T> {
    fn with_lock<F, R>(&self, f: F) -> Result<R, PoisonError<R>>
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut guard = match self.lock() {
            Ok(g) => g,
            // ⭐️ 修复: 将整个 PoisonError 结构体作为值接收 (e)
            Err(e) => {
                // .into_inner() 消费 (consume) PoisonError 并返回 MutexGuard<T>
                let mut poisoned_guard = e.into_inner();

                // 执行闭包修复操作
                let result = f(&mut poisoned_guard);

                // 返回一个新的 PoisonError (因为我们无法保证修复成功)
                return Err(PoisonError::new(result));
            }
        };

        let result = f(&mut guard);
        Ok(result)
    }
}
