
# dIndex

This repository is a rewrite of a previous design,
with a number of (currently undocumented) improvements
both in design and implementation.

Until I can find time to properly document the new changes,
the old readme is below and _mostly_ outlines the project
and goals.

One simplification is moving from a multi-binary structure to a single-binary.
The other is a focus on GUI components earlier in the dev process (see `http_client.rs`),
which I expect will make testing easier.

# FFI 

See `ffi-generators/readme.md`

# Dependencies

`libssl` version 1.1+

# Benchmarks

```
cargo bench
```

# Fuzzing


```
cargo install afl

cargo afl build --release --features fuzzer --bin dindex-fuzzer

sudo sh -c 'echo core >/proc/sys/kernel/core_pattern'
sudo sh -c 'echo 1 | tee /sys/devices/system/cpu/cpu*/online'
sudo sh -c 'echo performance | tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor'

AFL_SKIP_CPUFREQ=1 cargo afl fuzz -i ./fuzz-in/ -o ./target/url-fuzz-target target/release/dindex-fuzzer

```

# Example Use

```
TODO
```

# License

```
/**
 *  dIndex - a distributed, organic, mechanical index for everything
 *  Copyright (C) 2019  Jeffrey McAteer <jeffrey.p.mcateer@gmail.com>
 *  
 *  This program is free software; you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation; version 2 of the License only.
 * 
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 * 
 *  You should have received a copy of the GNU General Public License along
 *  with this program; if not, write to the Free Software Foundation, Inc.,
 *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
 */
```

Note the removal of the auto-upgrade clause: GPLv3 rights are not granted.

