use std::alloc::{Layout, alloc, dealloc};
use std::cell::RefCell;

use daken_rs::{KeyResult, RomajiInput};

thread_local! {
    static MATCHERS: RefCell<Vec<Option<RomajiInput>>> = const { RefCell::new(Vec::new()) };
}

#[unsafe(no_mangle)]
pub extern "C" fn alloc_bytes(len: usize) -> *mut u8 {
    if len == 0 {
        return std::ptr::null_mut();
    }

    let layout = Layout::array::<u8>(len).expect("valid allocation layout");
    unsafe { alloc(layout) }
}

#[unsafe(no_mangle)]
pub extern "C" fn dealloc_bytes(ptr: *mut u8, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }

    let layout = Layout::array::<u8>(len).expect("valid allocation layout");
    unsafe {
        dealloc(ptr, layout);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn matcher_new(ptr: *const u8, len: usize) -> u32 {
    if ptr.is_null() {
        return u32::MAX;
    }

    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    let Ok(target) = std::str::from_utf8(bytes) else {
        return u32::MAX;
    };

    MATCHERS.with(|matchers| {
        let mut matchers = matchers.borrow_mut();
        matchers.push(Some(RomajiInput::new(target)));
        (matchers.len() - 1) as u32
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn matcher_reset(id: u32) {
    with_matcher(id, |matcher| matcher.reset());
}

#[unsafe(no_mangle)]
pub extern "C" fn matcher_input(id: u32, codepoint: u32) -> u32 {
    let Some(key) = char::from_u32(codepoint) else {
        return 2;
    };

    with_matcher(id, |matcher| match matcher.input(key) {
        KeyResult::Accepted => 0,
        KeyResult::Completed => 1,
        KeyResult::Rejected => 2,
    })
    .unwrap_or(2)
}

#[unsafe(no_mangle)]
pub extern "C" fn matcher_is_completed(id: u32) -> u32 {
    with_matcher(id, |matcher| u32::from(matcher.is_completed())).unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn matcher_typed_len(id: u32) -> usize {
    with_matcher(id, |matcher| matcher.typed().len()).unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn matcher_next_key_mask(id: u32) -> u32 {
    with_matcher(id, |matcher| {
        matcher.next_keys().into_iter().fold(0u32, |mask, key| {
            if key.is_ascii_lowercase() {
                let offset = key as u32 - 'a' as u32;
                mask | (1 << offset)
            } else {
                mask
            }
        })
    })
    .unwrap_or(0)
}

fn with_matcher<T>(id: u32, f: impl FnOnce(&mut RomajiInput) -> T) -> Option<T> {
    MATCHERS.with(|matchers| matchers.borrow_mut().get_mut(id as usize)?.as_mut().map(f))
}
