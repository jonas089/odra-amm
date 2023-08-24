# odra_amm
[Speculative execution & gas estimation](https://github.com/jonas089/casperAMM/tree/master/speculative-execution)
## Usage
It's recommend to install 
[cargo-odra](https://github.com/odradev/cargo-odra) first.

### Build

```
$ cargo odra build
```
To build a wasm file, you need to pass the -b parameter. 
The result files will be placed in `${project-root}/wasm` directory.

```
$ cargo odra build -b casper
```
