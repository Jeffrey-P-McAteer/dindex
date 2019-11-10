
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

# Example Use

`dindex` is designed to be heavily personalized to suit your indexing needs.
The first thing most users should to is edit their `~/.dindex.toml` configuration
file to have some servers and ctypes. A ctype is an alias for key-value records
that allows you to specify search and publish values without having to type
a full record object.

## Config

Example config to search web records from any machine on your LAN as well as the `jmcateer.pw` server:

```
# ~/.dindex.toml

[[servers]]
uri = "multicast://239.255.29.224"
name = "LAN Multicast"

[[servers]]
uri = "tcp://jmcateer.pw"
name = "Jeffrey's Blog TCP"

[[ctypes]]
name = ":web"
key_names = ["title", "url", "description"]

```

## Querying

Now when you invoke `dindex` the following queries are identical:

```
dindex query :web 'title content'
dindex query :web 'title content' '(?i)http://'
```

```
dindex query '{"title": "title content"}'
dindex query '{"title": "title content", "url": "(?i)http://"}'
```

The first type is mostly what users will want to use interactively,
while the raw object type is great for integrating with shell scripts and other programs.

Note the use of `(?i)` in the regex: this makes the match case-insensitive. Config
flags to make this default may appear in the future, but the ideal (and unfinished)
strategy is to hook `rhai_scripts` to add custom logic during search creation.

Results are currently formated like this, however a future plan is to integrate
`rhai_scripts` to allow custom formatting if a record matches a known type:

```
=== Blog TCP ===
res = {"title": "Title", "url": "http://example.org", "description": "Lorem Ipsum description"}
res = {"description": "Description", "url": "http://url.com", "title": "Some title"}
res = {"url": "http://url.com", "title": "Some title", "description": "Description number 2"}
=== LAN Multicast ===
res = {"title": "Some title", "url": "http://url.com", "description": "Description number 2"}
```

## Publishing

Publishing works exactly like querying, but instead of a regex you supply a value.

```
dindex publish :web 'title content'
dindex publish :web 'title content' 'http://example.org'
```

```
dindex publish '{"title": "title content"}'
dindex publish '{"title": "title content", "url": "http://example.org"}'
```

## Listening

Listening also follows the semantics of querying, but it does not return old records
and instead blocks until the server receives a new record matching the query:

```
dindex listen :web 'title content'
```

After publishing a record the listener will output:

```
res = {"url": "http://url.com", "description": "Description number 2", "title": "title content"}
res = {"url": "http://url.com", "title": "title content", "description": "Description number 3"}
res = {"description": "Description number 4", "url": "http://url.com", "title": "title content"}

```

At the moment listening is a bit broken when using `udp` connections,
and there is work to be done to print the source of received records.


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
