use crate::OpenJDK;
use crate::SINGLETON;
use crate::UPCALLS;
use mmtk::util::opaque_pointer::*;
use mmtk::vm::ActivePlan;
use mmtk::Mutator;
use mmtk::Plan;
use std::marker::PhantomData;
use std::sync::Mutex;
use std::sync::MutexGuard;

pub struct SynchronizedMutatorIterator<'a> {
    _guard: MutexGuard<'a, ()>,
    start: bool,
    phantom: PhantomData<OpenJDK>,
}

impl<'a> Iterator for SynchronizedMutatorIterator<'a> {
    type Item = &'a mut Mutator<OpenJDK>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start {
            self.start = false;
            unsafe {
                ((*UPCALLS).reset_mutator_iterator)();
            }
        }
        let _guard = MUTATOR_ITERATOR_LOCK.lock().unwrap();
        unsafe {
            let m = ((*UPCALLS).get_next_mutator)();
            if m.is_null() {
                None
            } else {
                Some(&mut *m)
            }
        }
    }
}

pub struct VMActivePlan {}

impl ActivePlan<OpenJDK> for VMActivePlan {
    fn global() -> &'static dyn Plan<VM = OpenJDK> {
        SINGLETON.get_plan()
    }

    fn is_mutator(tls: VMThread) -> bool {
        unsafe { ((*UPCALLS).is_mutator)(tls) }
    }

    fn mutator(tls: VMMutatorThread) -> &'static mut Mutator<OpenJDK> {
        unsafe {
            let m = ((*UPCALLS).get_mmtk_mutator)(tls);
            &mut *m
        }
    }

    fn number_of_mutators() -> usize {
        unsafe { ((*UPCALLS).number_of_mutators)() }
    }

    fn mutators<'a>() -> impl Iterator<Item = &'a mut Mutator<OpenJDK>> {
        SynchronizedMutatorIterator {
            _guard: Self::global().base().mutator_iterator_lock.lock().unwrap(),
            start: true,
            phantom: PhantomData,
        }
        .into_iter()
    }
}

lazy_static! {
    pub static ref MUTATOR_ITERATOR_LOCK: Mutex<()> = Mutex::new(());
}
