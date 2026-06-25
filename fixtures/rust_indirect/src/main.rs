#![allow(dead_code)]
#![allow(clippy::black_box)]
#![allow(non_camel_case_types)]

use std::any::Any;
use std::cell::Cell;
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::fmt;
use std::future::Future;
use std::hint::black_box;
use std::io::{Cursor, Read};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use std::thread;

use libc::c_int;

static GLOBAL_SINK: AtomicU64 = AtomicU64::new(0x1234_5678_90ab_cdef);

type FnPtr = fn(u64, u8) -> u64;
type ExternCb = extern "C" fn(u64, u8) -> u64;
type MethodPtr = fn(&method_pointer_Receiver, u64, u8) -> u64;

fn main() {
    ziggy::fuzz!(|data: &[u8]| {
        harness_entry(data);
    });
}

#[inline(never)]
fn harness_entry(data: &[u8]) {
    if data.is_empty() {
        sink(0);
        return;
    }

    static PANIC_HOOK_SET: AtomicBool = AtomicBool::new(false);
    if !PANIC_HOOK_SET.swap(true, Ordering::SeqCst) {
        std::panic::set_hook(Box::new(|_| {}));
    }

    let mut cursor = ByteCursor::new(data);
    let mut state = (data.len() as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    let rounds = data.len().clamp(1, 96);

    redherings_plant_dead_branches(state, data[0]);
    redherings_touch_addrtaken();
    redherings_touch_samesig();
    redherings_touch_uninst();

    for _ in 0..rounds {
        let selector = cursor.next_u8();
        state = match selector % 44 {
            0 => fnptr_call(state, cursor.next_u8()),
            1 => fn_table_call(state, cursor.next_u8()),
            2 => fn_struct_field_call(state, cursor.next_u8()),
            3 => fn_option_pointer_call(state, cursor.next_u8()),
            4 => fn_enum_embedded_pointer_call(state, cursor.next_u8()),
            5 => method_pointer_call(state, cursor.next_u8()),
            6 => extern_c_callback_call(state, cursor.next_u8()),
            7 => callback_fnptr_parameter_call(state, cursor.next_u8()),
            8 => dyn_fn_parameter_call(state, cursor.next_u8()),
            9 => dyn_fn_boxed_call(state, cursor.next_u8()),
            10 => dyn_fnmut_boxed_call(state, cursor.next_u8()),
            11 => dyn_fnonce_boxed_call(state, cursor.next_u8()),
            12 => callback_registry_dyn_fn_call(state, cursor.next_u8()),
            13 => trait_object_vtable_call(state, cursor.next_u8()),
            14 => trait_object_slice_call(state, data),
            15 => trait_object_arc_call(state, cursor.next_u8()),
            16 => iterator_trait_object_next_call(state, data),
            17 => io_read_trait_object_call(state, data),
            18 => fmt_write_trait_object_call(state, cursor.next_u8()),
            19 => dyn_debug_fmt_call(state, cursor.next_u8()),
            20 => dyn_error_source_call(state, cursor.next_u8()),
            21 => dyn_any_typeid_call(state, cursor.next_u8()),
            22 => drop_dyn_trait_object_call(state, cursor.next_u8()),
            23 => dyn_future_poll_call(state, cursor.next_u8()),
            24 => raw_waker_vtable_call(state, cursor.next_u8()),
            25 => vecdeque_callback_dyn_call(state, cursor.next_u8()),
            26 => thread_spawn_boxed_fnonce_call(state, cursor.next_u8()),
            27 => trait_object_nested_container_call(state, cursor.next_u8()),
            28 => closure_to_fnptr_call(state, cursor.next_u8()),
            29 => hashmap_fnptr_dispatch_call(state, cursor.next_u8()),
            30 => transmute_fnptr_signature_call(state, cursor.next_u8()),
            31 => usize_laundered_fnptr_call(state, cursor.next_u8()),
            32 => inline_asm_indirect_call(state, cursor.next_u8()),
            33 => trait_upcast_supertrait_call(state, cursor.next_u8()),
            34 => panic_unwind_drop_call(state, cursor.next_u8()),
            35 => plt_libc_edge_call(state, cursor.next_u8()),
            36 => dlsym_resolved_call(state, cursor.next_u8()),
            37 => signal_handler_call(state, cursor.next_u8()),
            38 => oncelock_lazy_init_call(state, cursor.next_u8()),
            39 => staticinit_initarray_call(state, cursor.next_u8()),
            40 => redherings_generic_static_dispatch_call(state, cursor.next_u8()),
            41 => redherings_closure_monomorphized_call(state, cursor.next_u8()),
            42 => redherings_enum_match_direct_call(state, cursor.next_u8()),
            _ => redherings_sort_by_generic_callback_call(state, data),
        };
    }

    if data.starts_with(b"macro") {
        state = redherings_macro_generated_direct_call(state, data[0]);
    }

    sink(state);
}

struct ByteCursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> ByteCursor<'a> {
    #[inline(never)]
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    #[inline(never)]
    fn next_u8(&mut self) -> u8 {
        let b = self.data.get(self.pos % self.data.len()).copied().unwrap_or(0);
        self.pos = self.pos.wrapping_add(1);
        b
    }
}

#[inline(never)]
fn sink(v: u64) -> u64 {
    let mixed = black_box(v.rotate_left(7) ^ 0xa5a5_5a5a_dead_beef);
    GLOBAL_SINK.fetch_xor(mixed, Ordering::Relaxed);
    mixed
}

#[inline(never)]
fn fnptr_target_add(x: u64, b: u8) -> u64 {
    x.wrapping_add((b as u64).wrapping_mul(3)).rotate_left(3)
}

#[inline(never)]
fn fnptr_target_xor(x: u64, b: u8) -> u64 {
    x ^ ((b as u64) << 17) ^ 0x1111_2222_3333_4444
}

#[inline(never)]
fn fnptr_target_mul(x: u64, b: u8) -> u64 {
    x.wrapping_mul((b as u64).wrapping_add(1))
}

#[inline(never)]
fn fnptr_target_sub(x: u64, b: u8) -> u64 {
    x.wrapping_sub((b as u64).wrapping_mul(0x101))
}

#[inline(never)]
fn fnptr_call(x: u64, b: u8) -> u64 {
    let f: FnPtr = if b & 1 == 0 { fnptr_target_add } else { fnptr_target_xor };
    f(x, b)
}

static FN_TABLE: [FnPtr; 4] = [
    fnptr_target_add,
    fnptr_target_xor,
    fnptr_target_mul,
    fnptr_target_sub,
];

#[inline(never)]
fn fn_table_call(x: u64, b: u8) -> u64 {
    let f = FN_TABLE[(b as usize) & 3];
    f(x, b)
}

struct fn_struct_field_Dispatcher {
    callback: FnPtr,
    salt: u8,
}

#[inline(never)]
fn fn_struct_field_call(x: u64, b: u8) -> u64 {
    let dispatcher = fn_struct_field_Dispatcher {
        callback: if b & 2 == 0 { fnptr_target_mul } else { fnptr_target_sub },
        salt: b ^ 0x5a,
    };
    (dispatcher.callback)(x, dispatcher.salt)
}

#[inline(never)]
fn fn_option_pointer_call(x: u64, b: u8) -> u64 {
    let maybe: Option<FnPtr> = match b & 3 {
        0 => Some(fnptr_target_add),
        1 => Some(fnptr_target_xor),
        2 => Some(fnptr_target_mul),
        _ => None,
    };
    maybe.map(|f| f(x, b)).unwrap_or_else(|| x.rotate_right(5))
}

enum fn_enum_embedded_pointer_Node {
    Direct(FnPtr),
    Pair(FnPtr, FnPtr),
    Value(u64),
}

#[inline(never)]
fn fn_enum_embedded_pointer_call(x: u64, b: u8) -> u64 {
    let node = match b % 3 {
        0 => fn_enum_embedded_pointer_Node::Direct(fnptr_target_add),
        1 => fn_enum_embedded_pointer_Node::Pair(fnptr_target_xor, fnptr_target_sub),
        _ => fn_enum_embedded_pointer_Node::Value(b as u64),
    };
    match node {
        fn_enum_embedded_pointer_Node::Direct(f) => f(x, b),
        fn_enum_embedded_pointer_Node::Pair(f, g) => g(f(x, b), b.wrapping_add(1)),
        fn_enum_embedded_pointer_Node::Value(v) => x ^ v,
    }
}

struct method_pointer_Receiver {
    salt: u64,
}

impl method_pointer_Receiver {
    #[inline(never)]
    fn method_pointer_target_mix(&self, x: u64, b: u8) -> u64 {
        x.wrapping_add(self.salt).rotate_left((b & 31) as u32)
    }

    #[inline(never)]
    fn method_pointer_target_fold(&self, x: u64, b: u8) -> u64 {
        x ^ self.salt.rotate_right((b & 31) as u32)
    }
}

#[inline(never)]
fn method_pointer_call(x: u64, b: u8) -> u64 {
    let receiver = method_pointer_Receiver { salt: 0xfeed_face_cafe_babe };
    let method: MethodPtr = if b & 1 == 0 {
        method_pointer_Receiver::method_pointer_target_mix
    } else {
        method_pointer_Receiver::method_pointer_target_fold
    };
    method(&receiver, x, b)
}

#[inline(never)]
extern "C" fn extern_c_callback_target_left(x: u64, b: u8) -> u64 {
    x.rotate_left((b & 31) as u32) ^ 0xc001_c0de
}

#[inline(never)]
extern "C" fn extern_c_callback_target_right(x: u64, b: u8) -> u64 {
    x.rotate_right((b & 31) as u32) ^ 0xface_feed
}

#[inline(never)]
fn extern_c_callback_invoke(cb: ExternCb, x: u64, b: u8) -> u64 {
    cb(x, b)
}

#[inline(never)]
fn extern_c_callback_call(x: u64, b: u8) -> u64 {
    let cb: ExternCb = if b & 1 == 0 {
        extern_c_callback_target_left
    } else {
        extern_c_callback_target_right
    };
    extern_c_callback_invoke(cb, x, b)
}

#[inline(never)]
fn callback_fnptr_parameter_invoke(cb: FnPtr, x: u64, b: u8) -> u64 {
    cb(x, b)
}

#[inline(never)]
fn callback_fnptr_parameter_call(x: u64, b: u8) -> u64 {
    callback_fnptr_parameter_invoke(fnptr_target_xor, x, b)
}

#[inline(never)]
fn dyn_fn_parameter_target(x: u64, b: u8) -> u64 {
    x.wrapping_add((b as u64) << 9) ^ 0x7777_aaaa
}

#[inline(never)]
fn dyn_fn_parameter_invoke(cb: &dyn Fn(u64, u8) -> u64, x: u64, b: u8) -> u64 {
    cb(x, b)
}

#[inline(never)]
fn dyn_fn_parameter_call(x: u64, b: u8) -> u64 {
    let captured = (b as u64) << 24;
    let closure = |a: u64, c: u8| dyn_fn_parameter_target(a ^ captured, c);
    dyn_fn_parameter_invoke(&closure, x, b)
}

#[inline(never)]
fn dyn_fn_boxed_call(x: u64, b: u8) -> u64 {
    let salt = (b as u64).wrapping_mul(0x0101_0101);
    let cb: Box<dyn Fn(u64) -> u64> = if b & 1 == 0 {
        Box::new(move |v| v.wrapping_add(salt))
    } else {
        Box::new(move |v| v.rotate_left((salt & 31) as u32))
    };
    cb(x)
}

#[inline(never)]
fn dyn_fnmut_boxed_call(x: u64, b: u8) -> u64 {
    let counter = Cell::new((b as u64) | 1);
    let mut cb: Box<dyn FnMut(u64) -> u64> = Box::new(move |v| {
        let n = counter.get().wrapping_mul(3);
        counter.set(n);
        v ^ n
    });
    cb(x)
}

#[inline(never)]
fn dyn_fnonce_boxed_call(x: u64, b: u8) -> u64 {
    let salt = (b as u64).wrapping_mul(0x1f1f_1f1f);
    let cb: Box<dyn FnOnce(u64) -> u64> = Box::new(move |v| v.wrapping_sub(salt).rotate_right(7));
    cb(x)
}

#[inline(never)]
fn callback_registry_dyn_fn_target_a(x: u64) -> u64 {
    x.rotate_left(11)
}

#[inline(never)]
fn callback_registry_dyn_fn_target_b(x: u64) -> u64 {
    x.wrapping_mul(0x1000_0001)
}

#[inline(never)]
fn callback_registry_dyn_fn_call(x: u64, b: u8) -> u64 {
    let registry: Vec<Box<dyn Fn(u64) -> u64>> = vec![
        Box::new(callback_registry_dyn_fn_target_a),
        Box::new(callback_registry_dyn_fn_target_b),
        Box::new(move |v| v ^ ((b as u64) << 32)),
    ];
    registry[(b as usize) % registry.len()](x)
}

trait trait_object_Op: Send + Sync {
    fn trait_object_apply(&self, x: u64, b: u8) -> u64;
    fn trait_object_tag(&self) -> u64;
}

struct trait_object_Add {
    salt: u64,
}

struct trait_object_Rotate {
    salt: u64,
}

impl trait_object_Op for trait_object_Add {
    #[inline(never)]
    fn trait_object_apply(&self, x: u64, b: u8) -> u64 {
        x.wrapping_add(self.salt).wrapping_add(b as u64)
    }

    #[inline(never)]
    fn trait_object_tag(&self) -> u64 {
        0xaadd
    }
}

impl trait_object_Op for trait_object_Rotate {
    #[inline(never)]
    fn trait_object_apply(&self, x: u64, b: u8) -> u64 {
        x.rotate_left((b & 31) as u32) ^ self.salt
    }

    #[inline(never)]
    fn trait_object_tag(&self) -> u64 {
        0x7707
    }
}

#[inline(never)]
fn trait_object_vtable_call(x: u64, b: u8) -> u64 {
    let op: Box<dyn trait_object_Op> = if b & 1 == 0 {
        Box::new(trait_object_Add { salt: 0x1010 })
    } else {
        Box::new(trait_object_Rotate { salt: 0x2020 })
    };
    op.trait_object_apply(x, b) ^ op.trait_object_tag()
}

trait trait_object_SliceLike {
    fn trait_object_len(&self) -> usize;
    fn trait_object_byte(&self, idx: usize) -> u8;
}

impl trait_object_SliceLike for Vec<u8> {
    #[inline(never)]
    fn trait_object_len(&self) -> usize {
        self.len()
    }

    #[inline(never)]
    fn trait_object_byte(&self, idx: usize) -> u8 {
        self.get(idx % self.len().max(1)).copied().unwrap_or(0)
    }
}

impl trait_object_SliceLike for [u8; 8] {
    #[inline(never)]
    fn trait_object_len(&self) -> usize {
        self.len()
    }

    #[inline(never)]
    fn trait_object_byte(&self, idx: usize) -> u8 {
        self[idx % self.len()]
    }
}

#[inline(never)]
fn trait_object_slice_call(x: u64, data: &[u8]) -> u64 {
    let fallback = [1, 3, 3, 7, 5, 8, 13, 21];
    let object: Box<dyn trait_object_SliceLike> = if data.len() & 1 == 0 {
        Box::new(data.iter().copied().take(16).collect::<Vec<u8>>())
    } else {
        Box::new(fallback)
    };
    let idx = (x as usize) % object.trait_object_len().max(1);
    x ^ object.trait_object_byte(idx) as u64
}

#[inline(never)]
fn trait_object_arc_call(x: u64, b: u8) -> u64 {
    let op: Arc<dyn trait_object_Op> = if b & 1 == 0 {
        Arc::new(trait_object_Add { salt: 0x3333 })
    } else {
        Arc::new(trait_object_Rotate { salt: 0x4444 })
    };
    op.trait_object_apply(x, b)
}

#[inline(never)]
fn iterator_trait_object_next_call(x: u64, data: &[u8]) -> u64 {
    let mut iter: Box<dyn Iterator<Item = u8> + '_> = if data[0] & 1 == 0 {
        Box::new(data.iter().copied())
    } else {
        Box::new((0..data[0]).map(|v| v.wrapping_mul(3)))
    };

    let mut acc = x;
    for _ in 0..4 {
        acc ^= iter.next().unwrap_or(0) as u64;
        acc = acc.rotate_left(5);
    }
    acc
}

#[inline(never)]
fn io_read_trait_object_call(x: u64, data: &[u8]) -> u64 {
    let mut cursor = Cursor::new(data);
    let reader: &mut dyn Read = &mut cursor;
    let mut buf = [0u8; 8];
    let n = reader.read(&mut buf).unwrap_or(0);
    buf.iter().take(n).fold(x ^ n as u64, |acc, v| acc.wrapping_add(*v as u64))
}

struct fmt_write_trait_object_Sink {
    sum: u64,
}

impl fmt::Write for fmt_write_trait_object_Sink {
    #[inline(never)]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.as_bytes() {
            self.sum = self.sum.wrapping_mul(131).wrapping_add(*b as u64);
        }
        Ok(())
    }
}

#[inline(never)]
fn fmt_write_trait_object_call(x: u64, b: u8) -> u64 {
    let mut sink_obj = fmt_write_trait_object_Sink { sum: x };
    let writer: &mut dyn fmt::Write = &mut sink_obj;
    let _ = write!(writer, "fmt-write:{b:02x}:{x:016x}");
    sink_obj.sum
}

#[inline(never)]
fn dyn_debug_fmt_call(x: u64, b: u8) -> u64 {
    let left = (x, b);
    let right = [b, b.rotate_left(1), b.rotate_right(1), 0xaa];
    let debug_obj: &dyn fmt::Debug = if b & 1 == 0 { &left } else { &right };
    let rendered = format!("{debug_obj:?}");
    x ^ rendered.len() as u64
}

#[derive(Debug)]
struct dyn_error_source_Leaf;

impl fmt::Display for dyn_error_source_Leaf {
    #[inline(never)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "leaf")
    }
}

impl Error for dyn_error_source_Leaf {}

#[derive(Debug)]
struct dyn_error_source_Wrapper {
    inner: dyn_error_source_Leaf,
    code: u8,
}

impl fmt::Display for dyn_error_source_Wrapper {
    #[inline(never)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "wrapper-{}", self.code)
    }
}

impl Error for dyn_error_source_Wrapper {
    #[inline(never)]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.inner)
    }
}

#[inline(never)]
fn dyn_error_source_call(x: u64, b: u8) -> u64 {
    let wrapper = dyn_error_source_Wrapper {
        inner: dyn_error_source_Leaf,
        code: b,
    };
    let err: &dyn Error = &wrapper;
    let text_len = err.to_string().len() as u64;
    let source_len = err.source().map(|e| e.to_string().len() as u64).unwrap_or(0);
    x ^ text_len ^ source_len
}

#[inline(never)]
fn dyn_any_typeid_call(x: u64, b: u8) -> u64 {
    let tuple = (x, b, 0x99u8);
    let array = [b; 4];
    let any_obj: &dyn Any = if b & 1 == 0 { &tuple } else { &array };
    let is_tuple = any_obj.is::<(u64, u8, u8)>();
    x ^ if is_tuple { 0xaaaa } else { 0xbbbb }
}

trait drop_dyn_trait_Object {
    fn drop_dyn_trait_touch(&self, x: u64) -> u64;
}

struct drop_dyn_trait_Payload {
    salt: u64,
}

impl drop_dyn_trait_Object for drop_dyn_trait_Payload {
    #[inline(never)]
    fn drop_dyn_trait_touch(&self, x: u64) -> u64 {
        x ^ self.salt
    }
}

impl Drop for drop_dyn_trait_Payload {
    #[inline(never)]
    fn drop(&mut self) {
        sink(self.salt ^ 0xd0d0_d0d0);
    }
}

#[inline(never)]
fn drop_dyn_trait_object_call(x: u64, b: u8) -> u64 {
    let obj: Box<dyn drop_dyn_trait_Object> = Box::new(drop_dyn_trait_Payload {
        salt: (b as u64) << 40,
    });
    let out = obj.drop_dyn_trait_touch(x);
    drop(obj);
    out
}

#[inline(never)]
async fn dyn_future_async_target(x: u64, b: u8) -> u64 {
    x.wrapping_add((b as u64) << 48).rotate_left(9)
}

#[inline(never)]
fn dyn_future_poll_call(x: u64, b: u8) -> u64 {
    let mut future: Pin<Box<dyn Future<Output = u64>>> = Box::pin(dyn_future_async_target(x, b));
    let waker = raw_waker_make_waker();
    let mut cx = Context::from_waker(&waker);
    match future.as_mut().poll(&mut cx) {
        Poll::Ready(v) => v,
        Poll::Pending => x ^ 0xf00d,
    }
}

static RAW_WAKER_VTABLE_OBJECT: RawWakerVTable = RawWakerVTable::new(
    raw_waker_vtable_clone,
    raw_waker_vtable_wake,
    raw_waker_vtable_wake_by_ref,
    raw_waker_vtable_drop,
);

#[inline(never)]
unsafe fn raw_waker_vtable_clone(data: *const ()) -> RawWaker {
    sink(data as usize as u64 ^ 0x111);
    RawWaker::new(data, &RAW_WAKER_VTABLE_OBJECT)
}

#[inline(never)]
unsafe fn raw_waker_vtable_wake(data: *const ()) {
    sink(data as usize as u64 ^ 0x222);
}

#[inline(never)]
unsafe fn raw_waker_vtable_wake_by_ref(data: *const ()) {
    sink(data as usize as u64 ^ 0x333);
}

#[inline(never)]
unsafe fn raw_waker_vtable_drop(data: *const ()) {
    sink(data as usize as u64 ^ 0x444);
}

#[inline(never)]
fn raw_waker_make_waker() -> Waker {
    let raw = RawWaker::new(std::ptr::null(), &RAW_WAKER_VTABLE_OBJECT);
    unsafe { Waker::from_raw(raw) }
}

#[inline(never)]
fn raw_waker_vtable_call(x: u64, b: u8) -> u64 {
    let waker = raw_waker_make_waker();
    let cloned = waker.clone();
    waker.wake_by_ref();
    drop(cloned);
    x ^ ((b as u64) << 8)
}

#[inline(never)]
fn vecdeque_callback_dyn_call(x: u64, b: u8) -> u64 {
    let mut q: VecDeque<Box<dyn Fn(u64) -> u64>> = VecDeque::new();
    q.push_back(Box::new(|v| v.rotate_left(13)));
    q.push_back(Box::new(move |v| v ^ ((b as u64) << 56)));
    q.push_back(Box::new(fnptr_target_add_as_unary));
    let cb = q.remove((b as usize) % q.len()).unwrap();
    cb(x)
}

#[inline(never)]
fn fnptr_target_add_as_unary(x: u64) -> u64 {
    fnptr_target_add(x, 7)
}

#[inline(never)]
fn thread_spawn_boxed_fnonce_target(x: u64, b: u8) -> u64 {
    x.wrapping_add(0x5151_5151).rotate_right((b & 31) as u32)
}

#[inline(never)]
fn thread_spawn_boxed_fnonce_call(x: u64, b: u8) -> u64 {
    let cb: Box<dyn FnOnce() -> u64 + Send + 'static> = Box::new(move || thread_spawn_boxed_fnonce_target(x, b));
    let handle = thread::spawn(cb);
    handle.join().unwrap_or(x ^ 0x5151)
}

struct trait_object_nested_container_Holder {
    op: Box<dyn trait_object_Op>,
}

impl trait_object_nested_container_Holder {
    #[inline(never)]
    fn trait_object_nested_container_call(&self, x: u64, b: u8) -> u64 {
        self.op.trait_object_apply(x, b) ^ self.op.trait_object_tag()
    }
}

#[inline(never)]
fn trait_object_nested_container_call(x: u64, b: u8) -> u64 {
    let holder = trait_object_nested_container_Holder {
        op: if b & 1 == 0 {
            Box::new(trait_object_Add { salt: 0x9090 })
        } else {
            Box::new(trait_object_Rotate { salt: 0x8080 })
        },
    };
    holder.trait_object_nested_container_call(x, b)
}

#[inline(never)]
fn closure_to_fnptr_call(x: u64, b: u8) -> u64 {
    let f: FnPtr = if b & 1 == 0 {
        |v, c| v.wrapping_add((c as u64) << 3)
    } else {
        |v, c| v.rotate_left((c & 31) as u32)
    };
    f(x, b)
}

#[inline(never)]
fn hashmap_fnptr_dispatch_call(x: u64, b: u8) -> u64 {
    let mut table: HashMap<u8, FnPtr> = HashMap::new();
    table.insert(0, fnptr_target_add);
    table.insert(1, fnptr_target_xor);
    table.insert(2, fnptr_target_mul);
    table.insert(3, fnptr_target_sub);
    match table.get(&(b & 3)) {
        Some(f) => f(x, b),
        None => x.rotate_right(3),
    }
}

#[inline(never)]
fn transmute_fnptr_signature_target(x: u64, b: u8) -> u64 {
    x.wrapping_mul(0x9e37).wrapping_add(b as u64)
}

#[inline(never)]
fn transmute_fnptr_signature_call(x: u64, b: u8) -> u64 {
    let raw: *const () = transmute_fnptr_signature_target as *const ();
    let f: FnPtr = unsafe { core::mem::transmute(raw) };
    f(x, b)
}

#[inline(never)]
fn usize_laundered_fnptr_target(x: u64, b: u8) -> u64 {
    x.wrapping_sub((b as u64) << 5) ^ 0x2718_2818
}

#[inline(never)]
fn usize_laundered_fnptr_call(x: u64, b: u8) -> u64 {
    let addr = usize_laundered_fnptr_target as usize;
    let laundered = (addr ^ 0xff00) ^ 0xff00;
    let f: FnPtr = unsafe { core::mem::transmute(laundered as *const ()) };
    f(x, b)
}

#[cfg(target_arch = "x86_64")]
#[inline(never)]
fn inline_asm_indirect_call(x: u64, b: u8) -> u64 {
    let f: ExternCb = if b & 1 == 0 {
        extern_c_callback_target_left
    } else {
        extern_c_callback_target_right
    };
    let ret: u64;
    unsafe {
        core::arch::asm!(
            "mov r15, rsp",
            "and rsp, -16",
            "call {target}",
            "mov rsp, r15",
            target = in(reg) f,
            inout("rdi") x => _,
            inout("rsi") b as u64 => _,
            out("rax") ret,
            out("rcx") _, out("rdx") _, out("r8") _, out("r9") _,
            out("r10") _, out("r11") _, out("r15") _,
            out("xmm0") _, out("xmm1") _, out("xmm2") _, out("xmm3") _,
            out("xmm4") _, out("xmm5") _, out("xmm6") _, out("xmm7") _,
            out("xmm8") _, out("xmm9") _, out("xmm10") _, out("xmm11") _,
            out("xmm12") _, out("xmm13") _, out("xmm14") _, out("xmm15") _,
        );
    }
    ret
}

#[cfg(not(target_arch = "x86_64"))]
#[inline(never)]
fn inline_asm_indirect_call(x: u64, b: u8) -> u64 {
    let f: ExternCb = if b & 1 == 0 {
        extern_c_callback_target_left
    } else {
        extern_c_callback_target_right
    };
    f(x, b)
}

trait trait_upcast_Base {
    fn trait_upcast_base_tag(&self) -> u64;
}

trait trait_upcast_Derived: trait_upcast_Base {
    fn trait_upcast_derived_apply(&self, x: u64, b: u8) -> u64;
}

struct trait_upcast_Impl {
    salt: u64,
}

impl trait_upcast_Base for trait_upcast_Impl {
    #[inline(never)]
    fn trait_upcast_base_tag(&self) -> u64 {
        self.salt ^ 0x00c0_ffee
    }
}

impl trait_upcast_Derived for trait_upcast_Impl {
    #[inline(never)]
    fn trait_upcast_derived_apply(&self, x: u64, b: u8) -> u64 {
        x.wrapping_add(self.salt).rotate_left((b & 31) as u32)
    }
}

#[inline(never)]
fn trait_upcast_supertrait_call(x: u64, b: u8) -> u64 {
    let derived: Box<dyn trait_upcast_Derived> = Box::new(trait_upcast_Impl {
        salt: (b as u64) << 32,
    });
    let applied = derived.trait_upcast_derived_apply(x, b);
    let base: &dyn trait_upcast_Base = &*derived;
    applied ^ base.trait_upcast_base_tag()
}

struct panic_unwind_DropGuard {
    salt: u64,
}

impl Drop for panic_unwind_DropGuard {
    #[inline(never)]
    fn drop(&mut self) {
        sink(self.salt ^ 0x0bad_0bad_0bad_0bad);
    }
}

#[inline(never)]
fn panic_unwind_drop_call(x: u64, b: u8) -> u64 {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = panic_unwind_DropGuard { salt: (b as u64) << 16 };
        if b & 1 == 0 {
            panic!("panic_unwind intentional");
        }
        x.rotate_left(9)
    }));
    match result {
        Ok(v) => v,
        Err(_) => x ^ 0xdead_d00d_0000_0000,
    }
}

#[inline(never)]
fn plt_libc_edge_call(x: u64, b: u8) -> u64 {
    let a = unsafe { libc::abs(black_box(-(b as c_int))) } as u64;
    let s = c"reachability-plt-edge";
    let l = unsafe { libc::strlen(s.as_ptr()) } as u64;
    x ^ a.wrapping_mul(0x101) ^ l
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn dlsym_resolved_target(x: u64, b: u8) -> u64 {
    x.wrapping_add((b as u64) << 20) ^ 0x005d_5d5d_5d5d
}

#[inline(never)]
fn dlsym_resolved_call(x: u64, b: u8) -> u64 {
    #[cfg(unix)]
    unsafe {
        let handle = libc::dlopen(std::ptr::null(), libc::RTLD_NOW | libc::RTLD_GLOBAL);
        if !handle.is_null() {
            let sym = libc::dlsym(handle, c"dlsym_resolved_target".as_ptr());
            let out = if !sym.is_null() {
                let f: ExternCb = std::mem::transmute(sym);
                f(x, b)
            } else {
                x.rotate_right(13)
            };
            libc::dlclose(handle);
            return out;
        }
    }
    x.rotate_right(13)
}

static SIGNAL_HANDLER_FLAG: AtomicU64 = AtomicU64::new(0);

#[inline(never)]
extern "C" fn signal_handler_target(_sig: c_int) {
    SIGNAL_HANDLER_FLAG.fetch_add(0x1001, Ordering::SeqCst);
}

#[inline(never)]
fn signal_handler_call(x: u64, b: u8) -> u64 {
    #[cfg(unix)]
    unsafe {
        let h = signal_handler_target as extern "C" fn(c_int);
        libc::signal(libc::SIGUSR1, h as usize);
        libc::raise(libc::SIGUSR1);
    }
    x ^ SIGNAL_HANDLER_FLAG.load(Ordering::SeqCst) ^ (b as u64)
}

static ONCELOCK_CELL: OnceLock<u64> = OnceLock::new();

#[inline(never)]
fn oncelock_init_closure_target() -> u64 {
    sink(0x0ce0_0ce0_0ce0_0ce0);
    0x4242_4242_4242_4242
}

#[inline(never)]
fn oncelock_lazy_init_call(x: u64, b: u8) -> u64 {
    let v = ONCELOCK_CELL.get_or_init(oncelock_init_closure_target);
    x ^ *v ^ (b as u64)
}

static STATICINIT_RAN: AtomicBool = AtomicBool::new(false);

#[inline(never)]
extern "C" fn staticinit_ctor_body() {
    STATICINIT_RAN.store(true, Ordering::SeqCst);
}

#[used]
#[cfg_attr(any(target_os = "linux", target_os = "android"), link_section = ".init_array")]
#[cfg_attr(target_vendor = "apple", link_section = "__DATA,__mod_init_func")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XCU")]
static STATICINIT_INITARRAY: extern "C" fn() = {
    extern "C" fn staticinit_ctor() {
        staticinit_ctor_body();
    }
    staticinit_ctor
};

#[inline(never)]
fn staticinit_initarray_call(x: u64, b: u8) -> u64 {
    let ran = STATICINIT_RAN.load(Ordering::SeqCst);
    x ^ if ran { 0x1_0000 } else { 0 } ^ (b as u64)
}

trait redherings_StaticDispatch {
    fn redherings_static_apply(&self, x: u64, b: u8) -> u64;
}

struct redherings_StaticDispatchImpl;

impl redherings_StaticDispatch for redherings_StaticDispatchImpl {
    #[inline(never)]
    fn redherings_static_apply(&self, x: u64, b: u8) -> u64 {
        x.wrapping_add((b as u64) << 16)
    }
}

#[inline(never)]
fn redherings_generic_static_dispatch<T: redherings_StaticDispatch>(op: &T, x: u64, b: u8) -> u64 {
    op.redherings_static_apply(x, b)
}

#[inline(never)]
fn redherings_generic_static_dispatch_call(x: u64, b: u8) -> u64 {
    redherings_generic_static_dispatch(&redherings_StaticDispatchImpl, x, b)
}

#[inline(never)]
fn redherings_closure_monomorphized_target(x: u64, b: u8) -> u64 {
    x ^ ((b as u64) << 24)
}

#[inline(never)]
fn redherings_closure_monomorphized_call(x: u64, b: u8) -> u64 {
    let closure = |v: u64| redherings_closure_monomorphized_target(v, b);
    closure(x)
}

enum redherings_DirectEnum {
    Add,
    Xor,
    Rotate,
}

#[inline(never)]
fn redherings_enum_match_direct_target_add(x: u64, b: u8) -> u64 {
    x.wrapping_add(b as u64)
}

#[inline(never)]
fn redherings_enum_match_direct_target_xor(x: u64, b: u8) -> u64 {
    x ^ b as u64
}

#[inline(never)]
fn redherings_enum_match_direct_target_rotate(x: u64, b: u8) -> u64 {
    x.rotate_left((b & 31) as u32)
}

#[inline(never)]
fn redherings_enum_match_direct_call(x: u64, b: u8) -> u64 {
    let e = match b % 3 {
        0 => redherings_DirectEnum::Add,
        1 => redherings_DirectEnum::Xor,
        _ => redherings_DirectEnum::Rotate,
    };
    match e {
        redherings_DirectEnum::Add => redherings_enum_match_direct_target_add(x, b),
        redherings_DirectEnum::Xor => redherings_enum_match_direct_target_xor(x, b),
        redherings_DirectEnum::Rotate => redherings_enum_match_direct_target_rotate(x, b),
    }
}

#[inline(never)]
fn redherings_sort_by_generic_callback_call(x: u64, data: &[u8]) -> u64 {
    let mut v: Vec<u8> = data.iter().copied().take(16).collect();
    v.sort_by(|a, b| b.cmp(a));
    v.iter().fold(x, |acc, b| acc.wrapping_mul(33).wrapping_add(*b as u64))
}

macro_rules! redherings_macro_direct {
    ($x:expr, $b:expr) => {{
        redherings_macro_generated_direct_target($x, $b)
    }};
}

#[inline(never)]
fn redherings_macro_generated_direct_target(x: u64, b: u8) -> u64 {
    x ^ ((b as u64) << 4) ^ 0x00ff_00ff
}

#[inline(never)]
fn redherings_macro_generated_direct_call(x: u64, b: u8) -> u64 {
    redherings_macro_direct!(x, b)
}

#[inline(never)]
fn redherings_deadbranch_const(x: u64, b: u8) -> u64 {
    sink(x ^ ((b as u64) << 1) ^ 0xc0c0_c0c0)
}

#[inline(never)]
fn redherings_deadbranch_volatile(x: u64, b: u8) -> u64 {
    sink(x ^ ((b as u64) << 2) ^ 0xd0d0_d0d0)
}

#[inline(never)]
fn redherings_plant_dead_branches(x: u64, b: u8) {
    if false {
        black_box(redherings_deadbranch_const(x, b));
    }
    if black_box(false) {
        black_box(redherings_deadbranch_volatile(x, b));
    }
}

#[inline(never)]
fn redherings_addrtaken_a(x: u64, b: u8) -> u64 {
    sink(x ^ (b as u64) ^ 0x0a0a_0a0a)
}

#[inline(never)]
fn redherings_addrtaken_b(x: u64, b: u8) -> u64 {
    sink(x ^ (b as u64) ^ 0x0b0b_0b0b)
}

static REDHERINGS_UNCALLED_TABLE: [FnPtr; 2] = [redherings_addrtaken_a, redherings_addrtaken_b];

#[inline(never)]
fn redherings_touch_addrtaken() {
    let a = REDHERINGS_UNCALLED_TABLE[0] as usize;
    let b = REDHERINGS_UNCALLED_TABLE[1] as usize;
    black_box(a ^ b);
}

#[inline(never)]
fn redherings_samesig_x(x: u64, b: u8) -> u64 {
    sink(x ^ (b as u64) ^ 0x0111_0111)
}

#[inline(never)]
fn redherings_samesig_y(x: u64, b: u8) -> u64 {
    sink(x ^ (b as u64) ^ 0x0222_0222)
}

#[inline(never)]
fn redherings_touch_samesig() {
    let x: FnPtr = redherings_samesig_x;
    let y: FnPtr = redherings_samesig_y;
    black_box(x as usize ^ y as usize);
}

struct redherings_UninstWidget;

impl trait_object_Op for redherings_UninstWidget {
    #[inline(never)]
    fn trait_object_apply(&self, x: u64, b: u8) -> u64 {
        sink(x ^ (b as u64) ^ 0x0999_0999)
    }

    #[inline(never)]
    fn trait_object_tag(&self) -> u64 {
        0xdead
    }
}

#[inline(never)]
fn redherings_touch_uninst() {
    let apply: fn(&redherings_UninstWidget, u64, u8) -> u64 =
        <redherings_UninstWidget as trait_object_Op>::trait_object_apply;
    let tag: fn(&redherings_UninstWidget) -> u64 =
        <redherings_UninstWidget as trait_object_Op>::trait_object_tag;
    black_box(apply as usize ^ tag as usize);
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn unreachable_fnptr_dead(seed: u64) -> u64 {
    let f: FnPtr = if seed & 1 == 0 { fnptr_target_add } else { fnptr_target_xor };
    f(seed, seed as u8)
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn unreachable_trait_object_dead(seed: u64) -> u64 {
    let op: Box<dyn trait_object_Op> = if seed & 1 == 0 {
        Box::new(trait_object_Add { salt: seed })
    } else {
        Box::new(trait_object_Rotate { salt: seed })
    };
    op.trait_object_apply(seed, seed as u8)
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn unreachable_dyn_fn_dead(seed: u64) -> u64 {
    let cb: Box<dyn Fn(u64) -> u64> = Box::new(move |v| v ^ seed.rotate_left(3));
    cb(seed)
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn unreachable_raw_waker_dead(seed: u64) -> u64 {
    let waker = raw_waker_make_waker();
    let clone = waker.clone();
    drop(clone);
    seed ^ 0xdead_dead
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn unreachable_redherings_direct_dead(seed: u64) -> u64 {
    redherings_enum_match_direct_target_add(seed, seed as u8)
}

#[no_mangle]
pub extern "C" fn LLVMFuzzerTestOneInput(data: *const u8, size: usize) -> i32 {
    if !data.is_null() && size > 0 {
        let slice = unsafe { std::slice::from_raw_parts(data, size) };
        harness_entry(slice);
    }
    0
}
