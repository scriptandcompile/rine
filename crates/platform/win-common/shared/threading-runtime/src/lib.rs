//! Shared TLS slot management and wait execution helpers.
//!
//! This crate owns the mutable runtime behavior used by the Windows DLL
//! implementations while leaving the public waitable data structures in
//! `rine-types`.

use std::cell::RefCell;
use std::sync::{Condvar, LazyLock, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const INFINITE: u32 = 0xFFFF_FFFF;
const WAIT_OBJECT_0: u32 = 0x0000_0000;
const WAIT_TIMEOUT: u32 = 0x0000_0102;

const TLS_MIN_AVAILABLE: usize = 64;
const TLS_EXPANSION: usize = 1024;
const TLS_MAX_SLOTS: usize = TLS_MIN_AVAILABLE + TLS_EXPANSION;

static TLS_BITMAP: LazyLock<Mutex<[bool; TLS_MAX_SLOTS]>> =
    LazyLock::new(|| Mutex::new([false; TLS_MAX_SLOTS]));

thread_local! {
    static TLS_VALUES: RefCell<Vec<usize>> = RefCell::new(vec![0; TLS_MAX_SLOTS]);
}

pub trait MutexStateLike {
    fn owner(&self) -> Option<thread::ThreadId>;
    fn set_owner(&mut self, owner: Option<thread::ThreadId>);
    fn count(&self) -> u32;
    fn set_count(&mut self, count: u32);
}

pub struct ThreadWaitRef<'a> {
    pub completed: &'a (Mutex<bool>, Condvar),
}

pub struct ProcessWaitRef<'a> {
    pub completed: &'a (Mutex<bool>, Condvar),
}

pub struct EventWaitRef<'a> {
    pub signaled: &'a Mutex<bool>,
    pub condvar: &'a Condvar,
    pub manual_reset: bool,
}

pub struct MutexWaitRef<'a, S> {
    pub state: &'a Mutex<S>,
    pub condvar: &'a Condvar,
}

pub struct SemaphoreWaitRef<'a> {
    pub count: &'a Mutex<i32>,
    pub condvar: &'a Condvar,
}

pub enum WaitRef<'a, S> {
    Thread(ThreadWaitRef<'a>),
    Process(ProcessWaitRef<'a>),
    Event(EventWaitRef<'a>),
    Mutex(MutexWaitRef<'a, S>),
    Semaphore(SemaphoreWaitRef<'a>),
}

pub trait WaitableLike {
    type MutexState: MutexStateLike;

    fn as_wait_ref(&self) -> WaitRef<'_, Self::MutexState>;
}

pub fn tls_alloc() -> Option<u32> {
    let mut bitmap = TLS_BITMAP.lock().unwrap();
    for (index, slot) in bitmap.iter_mut().enumerate() {
        if !*slot {
            *slot = true;
            return Some(index as u32);
        }
    }
    None
}

pub fn tls_free(index: u32) -> bool {
    let idx = index as usize;
    if idx >= TLS_MAX_SLOTS {
        return false;
    }

    let mut bitmap = TLS_BITMAP.lock().unwrap();
    if !bitmap[idx] {
        return false;
    }

    bitmap[idx] = false;
    TLS_VALUES.with(|values| {
        let mut values = values.borrow_mut();
        if idx < values.len() {
            values[idx] = 0;
        }
    });

    true
}

pub fn tls_get_value(index: u32) -> usize {
    let idx = index as usize;
    if idx >= TLS_MAX_SLOTS {
        return 0;
    }

    TLS_VALUES.with(|values| values.borrow().get(idx).copied().unwrap_or(0))
}

pub fn tls_set_value(index: u32, value: usize) -> bool {
    let idx = index as usize;
    if idx >= TLS_MAX_SLOTS {
        return false;
    }

    TLS_VALUES.with(|values| {
        let mut values = values.borrow_mut();
        if idx < values.len() {
            values[idx] = value;
        }
    });

    true
}

pub fn wait_on<W>(waitable: &W, timeout_ms: u32) -> u32
where
    W: WaitableLike,
{
    match waitable.as_wait_ref() {
        WaitRef::Thread(thread) => wait_completion(thread.completed, timeout_ms),
        WaitRef::Process(process) => wait_completion(process.completed, timeout_ms),
        WaitRef::Event(event) => wait_event(
            event.signaled,
            event.condvar,
            event.manual_reset,
            timeout_ms,
        ),
        WaitRef::Mutex(mutex) => wait_mutex(mutex.state, mutex.condvar, timeout_ms),
        WaitRef::Semaphore(semaphore) => {
            wait_semaphore(semaphore.count, semaphore.condvar, timeout_ms)
        }
    }
}

fn wait_completion(completed: &(Mutex<bool>, Condvar), timeout_ms: u32) -> u32 {
    let (lock, condvar) = completed;
    let mut done = lock.lock().unwrap();
    if *done {
        return WAIT_OBJECT_0;
    }
    if timeout_ms == 0 {
        return WAIT_TIMEOUT;
    }
    if timeout_ms == INFINITE {
        while !*done {
            done = condvar.wait(done).unwrap();
        }
        return WAIT_OBJECT_0;
    }

    let deadline = Instant::now() + Duration::from_millis(timeout_ms as u64);
    while !*done {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return WAIT_TIMEOUT;
        }
        let result = condvar.wait_timeout(done, remaining).unwrap();
        done = result.0;
    }

    WAIT_OBJECT_0
}

fn wait_event(
    signaled: &Mutex<bool>,
    condvar: &Condvar,
    manual_reset: bool,
    timeout_ms: u32,
) -> u32 {
    let mut signaled = signaled.lock().unwrap();
    if *signaled {
        if !manual_reset {
            *signaled = false;
        }
        return WAIT_OBJECT_0;
    }
    if timeout_ms == 0 {
        return WAIT_TIMEOUT;
    }
    if timeout_ms == INFINITE {
        while !*signaled {
            signaled = condvar.wait(signaled).unwrap();
        }
        if !manual_reset {
            *signaled = false;
        }
        return WAIT_OBJECT_0;
    }

    let deadline = Instant::now() + Duration::from_millis(timeout_ms as u64);
    while !*signaled {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return WAIT_TIMEOUT;
        }
        let result = condvar.wait_timeout(signaled, remaining).unwrap();
        signaled = result.0;
    }

    if !manual_reset {
        *signaled = false;
    }

    WAIT_OBJECT_0
}

fn wait_mutex<S>(state: &Mutex<S>, condvar: &Condvar, timeout_ms: u32) -> u32
where
    S: MutexStateLike,
{
    let thread_id = thread::current().id();
    let mut state = state.lock().unwrap();

    if state.owner() == Some(thread_id) {
        let count = state.count() + 1;
        state.set_count(count);
        return WAIT_OBJECT_0;
    }

    if state.owner().is_none() {
        state.set_owner(Some(thread_id));
        state.set_count(1);
        return WAIT_OBJECT_0;
    }

    if timeout_ms == 0 {
        return WAIT_TIMEOUT;
    }

    if timeout_ms == INFINITE {
        while state.owner().is_some() {
            state = condvar.wait(state).unwrap();
        }
        state.set_owner(Some(thread_id));
        state.set_count(1);
        return WAIT_OBJECT_0;
    }

    let deadline = Instant::now() + Duration::from_millis(timeout_ms as u64);
    while state.owner().is_some() {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return WAIT_TIMEOUT;
        }
        let result = condvar.wait_timeout(state, remaining).unwrap();
        state = result.0;
    }

    state.set_owner(Some(thread_id));
    state.set_count(1);
    WAIT_OBJECT_0
}

fn wait_semaphore(count: &Mutex<i32>, condvar: &Condvar, timeout_ms: u32) -> u32 {
    let mut count = count.lock().unwrap();

    if *count > 0 {
        *count -= 1;
        return WAIT_OBJECT_0;
    }

    if timeout_ms == 0 {
        return WAIT_TIMEOUT;
    }

    if timeout_ms == INFINITE {
        while *count == 0 {
            count = condvar.wait(count).unwrap();
        }
        *count -= 1;
        return WAIT_OBJECT_0;
    }

    let deadline = Instant::now() + Duration::from_millis(timeout_ms as u64);
    while *count == 0 {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return WAIT_TIMEOUT;
        }
        let result = condvar.wait_timeout(count, remaining).unwrap();
        count = result.0;
    }

    *count -= 1;
    WAIT_OBJECT_0
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::{Duration, Instant};

    #[derive(Clone)]
    struct ThreadFixture {
        completed: Arc<(Mutex<bool>, Condvar)>,
    }

    #[derive(Clone)]
    struct ProcessFixture {
        completed: Arc<(Mutex<bool>, Condvar)>,
    }

    #[derive(Clone)]
    struct EventFixture {
        signaled: Arc<Mutex<bool>>,
        condvar: Arc<Condvar>,
        manual_reset: bool,
    }

    #[derive(Clone)]
    struct MutexFixture {
        state: Arc<Mutex<DummyMutexState>>,
        condvar: Arc<Condvar>,
    }

    #[derive(Clone)]
    struct SemaphoreFixture {
        count: Arc<Mutex<i32>>,
        condvar: Arc<Condvar>,
    }

    struct DummyMutexState {
        owner: Option<thread::ThreadId>,
        count: u32,
    }

    impl MutexStateLike for DummyMutexState {
        fn owner(&self) -> Option<thread::ThreadId> {
            self.owner
        }

        fn set_owner(&mut self, owner: Option<thread::ThreadId>) {
            self.owner = owner;
        }

        fn count(&self) -> u32 {
            self.count
        }

        fn set_count(&mut self, count: u32) {
            self.count = count;
        }
    }

    impl WaitableLike for ThreadFixture {
        type MutexState = DummyMutexState;

        fn as_wait_ref(&self) -> WaitRef<'_, Self::MutexState> {
            WaitRef::Thread(ThreadWaitRef {
                completed: &self.completed,
            })
        }
    }

    impl WaitableLike for ProcessFixture {
        type MutexState = DummyMutexState;

        fn as_wait_ref(&self) -> WaitRef<'_, Self::MutexState> {
            WaitRef::Process(ProcessWaitRef {
                completed: &self.completed,
            })
        }
    }

    impl WaitableLike for EventFixture {
        type MutexState = DummyMutexState;

        fn as_wait_ref(&self) -> WaitRef<'_, Self::MutexState> {
            WaitRef::Event(EventWaitRef {
                signaled: &self.signaled,
                condvar: &self.condvar,
                manual_reset: self.manual_reset,
            })
        }
    }

    impl WaitableLike for MutexFixture {
        type MutexState = DummyMutexState;

        fn as_wait_ref(&self) -> WaitRef<'_, Self::MutexState> {
            WaitRef::Mutex(MutexWaitRef {
                state: &self.state,
                condvar: &self.condvar,
            })
        }
    }

    impl WaitableLike for SemaphoreFixture {
        type MutexState = DummyMutexState;

        fn as_wait_ref(&self) -> WaitRef<'_, Self::MutexState> {
            WaitRef::Semaphore(SemaphoreWaitRef {
                count: &self.count,
                condvar: &self.condvar,
            })
        }
    }

    #[test]
    #[serial]
    fn tls_round_trip() {
        let index = tls_alloc().unwrap();
        assert_eq!(tls_get_value(index), 0);
        assert!(tls_set_value(index, 42));
        assert_eq!(tls_get_value(index), 42);
        assert!(tls_free(index));
        assert_eq!(tls_get_value(index), 0);
    }

    #[test]
    #[serial]
    fn sequential_tls_alloc_indices_differ() {
        let first = tls_alloc().unwrap();
        let second = tls_alloc().unwrap();
        assert_ne!(first, second);
        assert!(tls_free(first));
        assert!(tls_free(second));
    }

    #[test]
    #[serial]
    fn tls_free_unallocated_returns_false() {
        assert!(!tls_free(u32::MAX));
    }

    #[test]
    #[serial]
    fn tls_double_free_returns_false() {
        let index = tls_alloc().unwrap();
        assert!(tls_free(index));
        assert!(!tls_free(index));
    }

    #[test]
    #[serial]
    fn tls_out_of_range_index_returns_zero_or_false() {
        assert_eq!(tls_get_value(u32::MAX), 0);
        assert!(!tls_set_value(u32::MAX, 99));
    }

    #[test]
    #[serial]
    fn tls_realloc_reuses_freed_slot() {
        let first = tls_alloc().unwrap();
        assert!(tls_free(first));
        let second = tls_alloc().unwrap();
        assert!(second < u32::MAX);
        assert!(tls_free(second));
    }

    #[test]
    #[serial]
    fn tls_is_thread_local() {
        let index = tls_alloc().unwrap();
        assert!(tls_set_value(index, 7));

        let child = std::thread::spawn(move || {
            let seen = tls_get_value(index);
            assert!(tls_set_value(index, 9));
            (seen, tls_get_value(index))
        });

        assert_eq!(child.join().unwrap(), (0, 9));
        assert_eq!(tls_get_value(index), 7);
        assert!(tls_free(index));
    }

    fn make_event(manual_reset: bool, initial: bool) -> EventFixture {
        EventFixture {
            signaled: Arc::new(Mutex::new(initial)),
            condvar: Arc::new(Condvar::new()),
            manual_reset,
        }
    }

    #[test]
    fn event_auto_reset_is_consumed() {
        let event = make_event(false, true);
        assert_eq!(wait_on(&event, 0), WAIT_OBJECT_0);
        assert_eq!(wait_on(&event, 0), WAIT_TIMEOUT);
    }

    #[test]
    fn event_waits_until_signaled() {
        let event = make_event(true, false);
        let signaled = Arc::clone(&event.signaled);
        let condvar = Arc::clone(&event.condvar);
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(20));
            *signaled.lock().unwrap() = true;
            condvar.notify_all();
        });

        assert_eq!(wait_on(&event, 1000), WAIT_OBJECT_0);
    }

    #[test]
    fn event_initially_unsignaled_times_out() {
        let event = make_event(true, false);
        assert_eq!(wait_on(&event, 0), WAIT_TIMEOUT);
    }

    #[test]
    fn manual_reset_event_stays_signaled() {
        let event = make_event(true, true);
        assert_eq!(wait_on(&event, 0), WAIT_OBJECT_0);
        assert_eq!(wait_on(&event, 0), WAIT_OBJECT_0);
    }

    #[test]
    fn event_timeout_when_never_signaled() {
        let event = make_event(true, false);
        let start = Instant::now();
        assert_eq!(wait_on(&event, 50), WAIT_TIMEOUT);
        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(40));
        assert!(elapsed < Duration::from_millis(200));
    }

    fn make_mutex(initially_owned: bool) -> MutexFixture {
        let owner = initially_owned.then(|| thread::current().id());
        MutexFixture {
            state: Arc::new(Mutex::new(DummyMutexState {
                owner,
                count: u32::from(initially_owned),
            })),
            condvar: Arc::new(Condvar::new()),
        }
    }

    #[test]
    fn mutex_is_recursive_for_same_thread() {
        let mutex = make_mutex(true);
        assert_eq!(wait_on(&mutex, 0), WAIT_OBJECT_0);
        assert_eq!(mutex.state.lock().unwrap().count, 2);
    }

    #[test]
    fn mutex_times_out_when_held_elsewhere() {
        let mutex = make_mutex(false);
        let state = Arc::clone(&mutex.state);
        let condvar = Arc::clone(&mutex.condvar);
        let barrier = Arc::new((Mutex::new(false), Condvar::new()));
        let barrier_thread = Arc::clone(&barrier);

        let _holder = std::thread::spawn(move || {
            {
                let mut state = state.lock().unwrap();
                state.owner = Some(thread::current().id());
                state.count = 1;
            }
            let (ready, ready_condvar) = &*barrier_thread;
            *ready.lock().unwrap() = true;
            ready_condvar.notify_all();
            std::thread::sleep(Duration::from_millis(100));
            {
                let mut state = state.lock().unwrap();
                state.owner = None;
                state.count = 0;
            }
            condvar.notify_one();
        });

        let (ready, ready_condvar) = &*barrier;
        let mut ready = ready.lock().unwrap();
        while !*ready {
            ready = ready_condvar.wait(ready).unwrap();
        }

        assert_eq!(wait_on(&mutex, 10), WAIT_TIMEOUT);
    }

    #[test]
    fn mutex_unowned_acquires_immediately() {
        let mutex = make_mutex(false);
        assert_eq!(wait_on(&mutex, 0), WAIT_OBJECT_0);
    }

    #[test]
    fn mutex_wait_succeeds_when_released_by_other_thread() {
        let mutex = make_mutex(false);
        let state = Arc::clone(&mutex.state);
        let condvar = Arc::clone(&mutex.condvar);

        let child = std::thread::spawn(move || {
            {
                let mut state = state.lock().unwrap();
                state.owner = Some(thread::current().id());
                state.count = 1;
            }
            std::thread::sleep(Duration::from_millis(20));
            {
                let mut state = state.lock().unwrap();
                state.owner = None;
                state.count = 0;
            }
            condvar.notify_one();
        });
        child.join().unwrap();

        assert_eq!(wait_on(&mutex, 1000), WAIT_OBJECT_0);
    }

    fn make_semaphore(initial: i32) -> SemaphoreFixture {
        SemaphoreFixture {
            count: Arc::new(Mutex::new(initial)),
            condvar: Arc::new(Condvar::new()),
        }
    }

    #[test]
    fn semaphore_decrements_and_times_out() {
        let semaphore = make_semaphore(1);
        assert_eq!(wait_on(&semaphore, 0), WAIT_OBJECT_0);
        assert_eq!(wait_on(&semaphore, 0), WAIT_TIMEOUT);
    }

    #[test]
    fn semaphore_drains_to_zero() {
        let semaphore = make_semaphore(2);
        assert_eq!(wait_on(&semaphore, 0), WAIT_OBJECT_0);
        assert_eq!(wait_on(&semaphore, 0), WAIT_OBJECT_0);
        assert_eq!(wait_on(&semaphore, 0), WAIT_TIMEOUT);
        assert_eq!(*semaphore.count.lock().unwrap(), 0);
    }

    #[test]
    fn semaphore_release_wakes_waiter() {
        let semaphore = make_semaphore(0);
        let count = Arc::clone(&semaphore.count);
        let condvar = Arc::clone(&semaphore.condvar);
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(20));
            *count.lock().unwrap() += 1;
            condvar.notify_one();
        });

        assert_eq!(wait_on(&semaphore, 1000), WAIT_OBJECT_0);
    }

    #[test]
    fn semaphore_timeout_when_never_released() {
        let semaphore = make_semaphore(0);
        let start = Instant::now();
        assert_eq!(wait_on(&semaphore, 50), WAIT_TIMEOUT);
        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(40));
        assert!(elapsed < Duration::from_millis(200));
    }

    #[test]
    fn semaphore_multiple_releases_wake_multiple_waiters() {
        let semaphore = make_semaphore(0);
        let count = Arc::clone(&semaphore.count);
        let condvar = Arc::clone(&semaphore.condvar);

        let handles: Vec<_> = (0..3)
            .map(|_| {
                let waiter = semaphore.clone();
                std::thread::spawn(move || wait_on(&waiter, 2000))
            })
            .collect();

        std::thread::sleep(Duration::from_millis(30));
        {
            let mut count = count.lock().unwrap();
            *count += 3;
        }
        for _ in 0..3 {
            condvar.notify_one();
        }

        for handle in handles {
            assert_eq!(handle.join().unwrap(), WAIT_OBJECT_0);
        }
    }

    #[test]
    fn thread_wait_timeout_fires_when_not_completed() {
        let completed = Arc::new((Mutex::new(false), Condvar::new()));
        let thread = ThreadFixture { completed };
        let start = Instant::now();

        assert_eq!(wait_on(&thread, 50), WAIT_TIMEOUT);
        assert!(start.elapsed() >= Duration::from_millis(40));
    }

    #[test]
    fn process_completion_waits_on_shared_state() {
        let completed = Arc::new((Mutex::new(false), Condvar::new()));
        let process = ProcessFixture {
            completed: Arc::clone(&completed),
        };
        let exit_code = Arc::new(AtomicU32::new(259));
        let exit_code_thread = Arc::clone(&exit_code);

        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(20));
            exit_code_thread.store(0, Ordering::Release);
            let (done, condvar) = &*completed;
            *done.lock().unwrap() = true;
            condvar.notify_all();
        });

        assert_eq!(wait_on(&process, 1000), WAIT_OBJECT_0);
        assert_eq!(exit_code.load(Ordering::Acquire), 0);
    }

    #[test]
    fn thread_completion_waits_on_shared_state() {
        let completed = Arc::new((Mutex::new(false), Condvar::new()));
        let thread = ThreadFixture {
            completed: Arc::clone(&completed),
        };

        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(20));
            let (done, condvar) = &*completed;
            *done.lock().unwrap() = true;
            condvar.notify_all();
        });

        assert_eq!(wait_on(&thread, 1000), WAIT_OBJECT_0);
    }
}
