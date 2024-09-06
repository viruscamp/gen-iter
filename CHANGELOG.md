# version 0.4.1
* add support for immovable coroutine (self-referenced)
* change the varialble name `gen` as it will be a keyword in edition2024

# version 0.4.0
* follow [rename Generator to Coroutine](https://github.com/rust-lang/rust/pull/116958) and fix #10

# version 0.3
* made the crate no_std compatible (#5)
* added struct GenIterReturn and macro gen_iter_return! to iterate over a generator and get the return value (#6)

# version 0.2.1
* added `move` varient of gen-iter (#4)

# version 0.2
* updated to latest rust generator syntax

# version 0.1.2
* impl `From<Generator>` for GenIter
* added `gen_iter!` convienence macro

# version 0.1.1
* fixed documentation link

# version 0.1.0
* inital release
