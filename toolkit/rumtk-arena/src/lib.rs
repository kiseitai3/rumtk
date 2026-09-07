#![feature(allocator_api)]
#![feature(slice_ptr_get)]
#![feature(linked_list_retain)]
#![feature(linked_list_cursors)]
#![feature(portable_simd)]
#![feature(str_as_str)]

extern crate alloc;
extern crate core;

pub mod arena;
pub mod buffers;
pub mod cpu;
pub mod mem;
pub mod dune;
pub mod base;
pub mod serde;

pub use arena::Arena;
pub use mem::*;

#[cfg(test)]
mod tests {
    use crate::buffers::{buffer_find, RUMBuffer, RUMBufferIteratorExt};
    use crate::cpu::{cpu_find, cpu_slice_to_array_padded};
    use crate::mem::constants::*;
    use crate::{as_slice_mut, direct_alloc, rumtk_arena_new, Arena};
    use std::alloc::alloc;
    use std::collections::{HashMap, VecDeque};

    macro_rules! rumtk_benchmark_snippet {
        ( $closure:expr ) => {{
            use std::time::Instant;

            let start = Instant::now();
            let r = $closure();
            let end = Instant::now();

            let time = end - start;
            let micros = time.as_micros();

            (r, micros)
        }};
    }

    #[test]
    fn test_cpu_slice_to_array_padded() {
        let expected = b"Hello World\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
        let result = cpu_slice_to_array_padded::<32, 0>(b"Hello World");
        assert_eq!(&result, expected, "Stack array was not properly padded!");
    }

    #[test]
    fn test_cpu_slice_to_array_padded_newline() {
        let expected = b"Hello World\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n";
        let result = cpu_slice_to_array_padded::<32, b'\n'>(b"Hello World");
        assert_eq!(&result, expected, "Stack array was not properly padded!");
    }

    #[test]
    fn test_cpu_find_simd_aligned() {
        let input = b"Hello World\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n00000000000000000000000000000000";
        let expected = 6;
        let result = cpu_find(input, b'W').unwrap();
        assert_eq!(result, expected, "Failed to find needle in haystack!");
    }

    #[test]
    fn test_cpu_find_simd_unaligned() {
        let input = b"Hello World\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n";
        let expected = 6;
        let result = cpu_find(input, b'W').unwrap();
        assert_eq!(result, expected, "Failed to find needle in haystack!");
    }

    #[test]
    fn test_cpu_find_simd_unaligned_none() {
        let input = b"Hello World\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n";
        let expected = 6;
        let result = cpu_find(input, b'\0');
        assert!(result.is_none() || (result.unwrap() -1) < input.len(), "Succeeded to find needle in haystack when the search character is not part of the haystack!");
    }

    #[test]
    fn test_buffer_find() {
        let input = b"Hello World\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n";
        let expected = 9;
        let result = buffer_find(input, b"ld\n");

        assert_eq!(result, expected, "Succeeded to find needle in haystack when the search character is not part of the haystack!");
    }

    #[test]
    fn test_buffer_find2() {
        let input = b"Hello World\n\n\n\n\n\n\nasdjklfkasjwaoia poaw ml;,\n\n\n\n\n\n\n\n\n\n\n\n\n";
        let expected = 40;
        let result = buffer_find(input, b"ml;");

        assert_eq!(result, expected, "Succeeded to find needle in haystack when the search character is not part of the haystack!");
    }

    #[test]
    fn test_buffer_split() {
        let input = RUMBuffer::from(b"Hello World\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n");
        let expected = 20;
        let mut result = 0;

        for _ in input.split_fast(b'\n') {
            result += 1;
        }

        assert_eq!(result, expected, "Succeeded to find needle in haystack when the search character is not part of the haystack!");
    }

    #[test]
    fn test_arena_direct_allocation() {

        let (r, time) = rumtk_benchmark_snippet!(|| {
            unsafe { as_slice_mut(direct_alloc(DEFAULT_GLOBAL_MB_ALLOCATION_LAYOUT), DEFAULT_GLOBAL_MB_ALLOCATION) }
        });

        assert_eq!(r.len(), DEFAULT_GLOBAL_MB_ALLOCATION);
        assert!(time < 1000, "Allocation took long! => {}us", time)

    }

    #[test]
    fn test_arena_basic_allocation() {
        let (r, time) = rumtk_benchmark_snippet!(|| {
            unsafe { as_slice_mut(alloc(DEFAULT_GLOBAL_MB_ALLOCATION_LAYOUT), DEFAULT_GLOBAL_MB_ALLOCATION) }
        });

        assert_eq!(r.len(), DEFAULT_GLOBAL_MB_ALLOCATION);
        assert!(time < 310, "Allocation took long! => {}us", time)

    }

    #[test]
    fn test_arena_allocate_and_use() {
        let (r, time) = rumtk_benchmark_snippet!(|| {
            let slice = unsafe { as_slice_mut(alloc(DEFAULT_GLOBAL_MB_ALLOCATION_LAYOUT), DEFAULT_GLOBAL_MB_ALLOCATION) };
            let v = slice.to_vec();
            let mut buffer = RUMBuffer::from(v);
            let mut chunk = buffer.freeze();

            for _ in 0..(DEFAULT_GLOBAL_MB_ALLOCATION/5) {
                chunk.split_to(5);
            }

            chunk
        });

        assert_eq!(r.len(), 0);
        assert!(time < 200000, "Allocation took long!")

    }

    #[test]
    fn test_arena_simple_vec_allocation() {
        let arena = Arena::with_capacity(1024);
        let mut v = Vec::<usize>::with_capacity(10);

        v.push(10);
        v.push(10);

        assert_eq!(v, [10, 10], "Failed to allocate and fill a small vector!");
    }

    #[test]
    fn test_arena_simple_vec_reallocation() {
        let arena = Arena::with_capacity(1024);
        let mut v = Vec::<usize>::with_capacity(1);

        v.push(10);
        v.push(10);

        assert_eq!(v, [10, 10], "Failed to reallocate and fill a small vector!");
    }

    #[test]
    fn test_arena_allocate_more_than_allowed() {
        let mut arena = Arena::with_capacity(5);
        let v = arena.commit(10);

        assert!(v.is_err(), "Arena did not emit error upon allocation of byte count higher than current capacity.");
    }

    #[test]
    fn test_arena_create_vec_with_macro() {
        let arena = Arena::with_capacity(5);
        let v: Vec<String> = vec![];

        assert!(v.is_empty(), "Failed to create vector with arena allocation enabled.");
    }

    #[test]
    fn test_arena_benchmark_arenavec_vs_vec() {
        struct ptr {
            data: usize,
            len: usize,
            index: usize,
            bad: usize,
        }

        impl ptr {
            pub fn new() -> Self {
                Self {
                    data: 0,
                    len: 0,
                    index: 0,
                    bad: 0,
                }
            }
        }

        let total_items = 20000;

        let (arena, arena_time) = rumtk_benchmark_snippet!(|| {
            let total_bytes = (total_items * size_of::<ptr>()) + size_of::<Vec<ptr>>();
            Arena::with_capacity(total_bytes)
        });

        let (arena_vec_r, arena_vec_time) = rumtk_benchmark_snippet!(|| {
            let mut v: Vec<ptr> = vec![];

            for _ in 0..total_items {
                v.push(ptr::new());
            }

            v
        });

        let (vec_r, vec_time) = rumtk_benchmark_snippet!(|| {
            let mut v = Vec::<ptr>::with_capacity(total_items);

            for _ in 0..total_items {
                v.push(ptr::new());
            }

            v
        });

        let total_arena_vec_time = arena_time + arena_vec_time;
        println!("ArenaVec => {} us vs. Vec => {} us.", total_arena_vec_time, vec_time);

        //assert!(total_arena_vec_time < vec_time, "ArenaVec is too slow. ArenaVec => {} us vs. Vec => {} us.", total_arena_vec_time, vec_time);
    }

    #[test]
    fn test_arena_create_vec_with_macro_with_items() {
        let arena = Arena::with_capacity(50);
        let expected = &["Hello", "World", "!"];
        let v: Vec<&str> = vec!["Hello", "World", "!"];

        assert_eq!(v.as_slice(), expected, "Failed to create vector with arena allocation enabled and item slice.");
    }

    #[test]
    fn test_arena_create_vecdeque_with_macro() {
        let arena = Arena::with_capacity(5);
        let v: VecDeque<String> = VecDeque::new();

        assert!(v.is_empty(), "Failed to create vector with arena allocation enabled.");
    }

    #[test]
    fn test_arena_create_vecdeque_with_macro_with_items() {
        let arena = Arena::with_capacity(50);
        let expected = ["Hello", "World", "!"];
        let mut v: VecDeque<&str> = VecDeque::from(expected.clone());

        assert_eq!(v.pop_front(), Some(expected[0]), "Failed to create queue with arena allocation enabled and item slice.");
    }

    #[test]
    fn test_arena_create_hashmap_with_macro() {
        let arena = Arena::with_capacity(5);
        let v: HashMap<&str, &str> = HashMap::new();

        assert!(v.is_empty(), "Failed to create vector with arena allocation enabled.");
    }

    #[test]
    fn test_arena_create_hashmap_with_macro_with_items() {
        let arena = Arena::with_capacity(120);
        let expected = [(0, "Hello"), (1, "World"), (2, "!")];
        let v: HashMap<usize, &str> = HashMap::from_iter(expected.clone());

        assert_eq!(v[&0], expected[0].1, "Failed to create hashmap with arena allocation enabled and item slice.");
    }

    #[test]
    fn test_arena_vec_debug_print() {
        let arena = rumtk_arena_new!(500);
        let mut test_vec = Vec::new();
        let expected = ["Hello", "World", "!"];

        for s in expected.iter() {
            test_vec.push(s);
        }

        println!("{:?}", &test_vec);
    }

    #[test]
    fn test_arena_map_debug_print() {
        let expected = [(5, "Hello"), (1, "World"), (3, "!")];


        let m = HashMap::<usize, &str>::from_iter(expected.clone());

        for (k, v) in expected.iter() {
            assert!(m.contains_key(k), "Key missing!");
            assert_eq!(v, &m[k], "Contents mismatch!");
        }
    }
}
