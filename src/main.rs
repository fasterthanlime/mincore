use libc::{c_void, mincore};
use memmap2::Mmap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::mem::forget;

fn main() -> Result<(), Box<dyn Error>> {
    let file = unsafe { Mmap::map(&File::open(&env::args().nth(1).expect("Need an file"))?)? };

    let pagesize = 16384;
    let len = file.len();
    let pages = (file.len() + pagesize - 1) / pagesize;

    println!("Size {} bytes Pages {}", len, pages);
    let file_ptr = file.as_ptr();

    let mut answer: Vec<u8> = Vec::with_capacity(pages);
    let answer_ptr = answer.as_mut_ptr();
    forget(answer);

    let answer: Vec<u8>;
    unsafe {
        _ = mincore(file_ptr as *mut c_void, len, answer_ptr as _);
        answer = Vec::from_raw_parts(answer_ptr, pages, pages);
    }

    let mut sum = 0;
    for x in answer {
        sum += (x & 0x1) as usize;
    }
    println!(
        "Pages in cache {}/{} ({:.2}%)",
        sum,
        pages,
        (sum as f64 / pages as f64) * 100f64
    );

    Ok(())
}
