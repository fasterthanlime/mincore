use libc::mincore;
use memmap2::Mmap;
use std::env;
use std::error::Error;
use std::fs::File;

fn main() -> Result<(), Box<dyn Error>> {
    // keep unsafe blocks small - do our safe work outside of it.
    let path = env::args().nth(1).expect("Need an file");
    let file = File::open(&path)?;
    // even though `file` is shadowed, it lives on - you can be sure because
    // `Mmap` lives only as long as the underlying `File`. It would definitely
    // complain
    let file = unsafe { Mmap::map(&file)? };

    // not all platforms have 16K pages! on Linux x86_64 it's 4K and the
    // `mincore` call wrote out of bounds, see
    // <https://github.com/fasterthanlime/mincore/issues/1>
    let page_size = page_size::get();

    let len = file.len();
    // `div_ceil` isn't stable yet, but I believe that's what we're doing
    // here: <https://doc.rust-lang.org/stable/std/primitive.usize.html#method.div_ceil>
    let pages = (file.len() + page_size - 1) / page_size;

    println!("{path} is {len} bytes, Pages {pages}");

    // Unless you've profiled your code and zero-initialization is your
    // bottleneck, this will do fine. If you really need uninitialized
    // memory, look for MaybeUninit, see
    // <https://doc.rust-lang.org/stable/std/mem/union.MaybeUninit.html>
    let mut answer = vec![0u8; pages];

    // There's absolutely no need to forget the vector, or re-assemble it. The C
    // function isn't "passed an array" (or a Vec), just an address where to write
    // the result. We own `answer`, so we can let `mincore` borrow it for a bit.
    //
    // `as _` is a convenient way to cast to "whatever type is needed". Here
    // it's doing `*const u8` to `*mut c_void`
    //
    // note that `libc::mincore` wants a `*mut c_void` but I sure hope it
    // doesn't mutate its `addr` argument. here we're casting from a `*const T`
    // to a `*mut T` which is a big no-no, but I'm fairly sure `libc::mincore`
    // just has the wrong prototype.
    let ret = unsafe { mincore(file.as_ptr() as _, len, answer.as_mut_ptr()) };
    if ret != 0 {
        // error handling is always a good idea. if you were using MaybeUninit,
        // you'd need to only "assume_init" if mincore returned successfully,
        // otherwise that's UB.
        panic!("mincore failed with error {}", ret);
    }

    // showing off binary literals (`0b00101010010`) just for fun, and
    // sum, also just for fun.
    let in_cache: usize = answer.iter().map(|x| (x & 0b1) as usize).sum();

    // showing off `_` being ignored in number literals, easier to read than
    // `100f64`
    let percent_cached = (in_cache as f64 / pages as f64) * 100_f64;
    println!("Pages in cache {in_cache}/{pages} ({percent_cached:.2}%)",);

    Ok(())
}
