// TODO Copyright Header

//! KMutex thing

use kqueue::KQueue;
use core::intrinsics::size_of;
use mm::alloc::request_slab_allocator;
use core::fmt;
use core::fmt::{Show, Formatter};
use core::cell::*;
use core::ptr::*;

pub fn init_stage1() {
    request_slab_allocator("KMutex allocator", unsafe { size_of::<KMutex>() as u32 });
}

pub fn init_stage2() {}

pub struct KMutex {
    name : &'static str,
    held : Cell<bool>,
    queue : UnsafeCell<KQueue>,
    //no_copy : core::kinds::marker::NoCopy,
}

impl KMutex {
    pub fn new(name: &'static str) -> KMutex {
        KMutex { name : name, held : Cell::new(false), queue : UnsafeCell::new(KQueue::new()) }
    }

    /// Obtain the lock, waiting until it is freed. Note that there are no ordering/fairness
    /// gaurentees on who gets a lock when it is contested.
    pub fn lock(&self) {
        dbg!(debug::SCHED, "locking {} for {} of {}", self, current_thread!(), current_proc!());
        while self.held.get() {
            unsafe { self.queue.get().as_mut().expect("Kmutex queue cannot be null").wait(false) };
        }
        self.held.set(true);
        return;
    }

    /// Returns true if we got the lock, False if we didn't because of being canceled.
    pub fn lock_cancelable(&self) -> bool {
        dbg!(debug::SCHED, "cancelable locking {} for {} of {}", self, current_thread!(), current_proc!());
        while self.held.get() {
            ;
            if unsafe { !self.queue.get().as_mut().expect("Kmutex queue cannot be null").wait(false) } {
                return false;
            }
        }
        self.held.set(true);
        return true;
    }

    pub fn try_lock(&self) -> bool {
        if !self.held.get() {
            dbg!(debug::SCHED, "locking {} for {} of {}", self, current_thread!(), current_proc!());
            self.held.set(true);
            true
        } else {
            dbg!(debug::SCHED, "locking {} for {} of {} failed", self, current_thread!(), current_proc!());
            false
        }
    }

    pub fn unlock(&self) {
        dbg!(debug::SCHED, "unlocking {} for {} of {}", self, current_thread!(), current_proc!());
        assert!(self.held.get());
        self.held.set(false);
        unsafe { self.queue.get().as_mut().expect("Kmutex queue cannot be null")}.signal();
    }
}

impl Show for KMutex {
    fn fmt(&self, f : &mut Formatter) -> fmt::Result {
        write!(f, "KMutex '{}' {{ held: {}, waiters: {} }}", self.name, self.held.get(),
                unsafe { self.queue.get().as_mut().expect("Kmutex queue cannot be null")}.len())
    }
}