# zcp (zero-copy cp)

# Why?

I initially wanted to see if I could make a faster (and easier to use) tool for flashing drives than `dd`.
In order to acheive this, we use the `sendfile64` syscall, which allows for copying data from one file descriptor to another completely in kernel space.
This is opposed to calling `read` and `write` repeatedly, which copies data in and out of user space.
As of writing this, when copying a file to a block device, both `cp` and `dd` employ the method of repeatedly calling `read` and `write`.
You can also use this as a general replacement for `cp` in cases where `cp` is otherwise inefficent (run your own benchmarks!)

In my personal (non-rigorous) benchmarks, it seems to match dd in terms of time spent (when an optimal block size is found and used).
Even so, I consider this project a success, as I'm able to get good speeds without messing with dd options (and finding an optimal blocksize), and theoretically wasting less CPU time on context switching.
I want to collect more data, so please open issues with your results :)

# Usage

```bash
zcp <source> <destination>
```

# Credits

Initial inspiration from:
https://jvns.ca/blog/2016/01/23/sendfile-a-new-to-me-system-call/
https://blog.plenz.com/2014-04/so-you-want-to-write-to-a-file-real-fast.html

ioctl for getting block device size in Rust:
http://syhpoon.ca/posts/how-to-get-block-device-size-on-linux-with-rust/
