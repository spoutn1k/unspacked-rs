# `unspacked-rs`

## What ?

This tool is meant to accelerate execution of `spack`-ed scripts by replacing `spack` calls by their equivalent in environment manipulation.

## Why ?

Because `spack` is *slow*. For instance the following:
```sh
. /spack/share/spack/setup-env.sh

echo "Loading packages ..."
spack load --first trilinos
spack load --first jsoncpp
echo "Done !"

spack load --list
```
Takes more than 10s to run while the `unspacked` equivalent:
```sh
load_9eed1b0e789b80782838c7f85171514ff2070e49f9db023fe402d0bb63c3bfe7() {
# Compiled version of 'spack load --sh --first trilinos'
export ACLOCAL_PATH=/spack/opt/spack/linux-ubuntu20.04-x86_64/gcc-9.3.0/libx...
export BOOST_ROOT=/spack/opt/spack/linux-ubuntu20.04-x86_64/gcc-9.3.0/boost-...
export CMAKE_PREFIX_PATH=/spack/opt/spack/linux-ubuntu20.04-x86_64/gcc-9.3.0...
export LD_LIBRARY_PATH=/spack/opt/spack/linux-ubuntu20.04-x86_64/gcc-9.3.0/t...
export MANPATH=/spack/opt/spack/linux-ubuntu20.04-x86_64/gcc-9.3.0/mpich-3.4...
export PATH=/spack/opt/spack/linux-ubuntu20.04-x86_64/gcc-9.3.0/trilinos-13....
export PKG_CONFIG_PATH=/spack/opt/spack/linux-ubuntu20.04-x86_64/gcc-9.3.0/s...
export SPACK_LOADED_HASHES=an5qvws33abtt6ggtrdqyn2za5qm7uro:u4caraxm24o23syf...
}

load_dc9bc243e421ca740f39ba3dd705f5298f1c20490a0ec9bb59e406b81d1f7181() {
# Compiled version of 'spack load --sh --first jsoncpp'
export CMAKE_PREFIX_PATH=/spack/opt/spack/linux-ubuntu20.04-x86_64/gcc-9.3.0...
export LD_LIBRARY_PATH=/spack/opt/spack/linux-ubuntu20.04-x86_64/gcc-9.3.0/j...
export PKG_CONFIG_PATH=/spack/opt/spack/linux-ubuntu20.04-x86_64/gcc-9.3.0/j...
export SPACK_LOADED_HASHES=rftwq6kvnmmpt73kkbgzkfknct35laj5:an5qvws33abtt6gg...
}

echo "Loading packages ..."
load_9eed1b0e789b80782838c7f85171514ff2070e49f9db023fe402d0bb63c3bfe7
load_dc9bc243e421ca740f39ba3dd705f5298f1c20490a0ec9bb59e406b81d1f7181
echo "Done !"
```
Takes less than a hundredth of one, on the same machine.

```
$ time source ~/spacked.sh 
Loading packages ...
Done !
==> 17 loaded packages
[...]

real	0m11.907s
user	0m11.372s
sys	0m0.452s
$ spack unload
$ time source ~/unspacked.sh 
Loading packages ...
Done !

real	0m0.007s
user	0m0.007s
sys	0m0.000s
$ spack load --list
==> 17 loaded packages
[...]
```

On a single system where you load spack once this is just a few seconds, but when working with spack in temporary environments, having to repeat the load calls leads to a lot of wasted time.

## How ?

This crate makes use of the `conch_parser` crate to interpret `shell` scripts, extract `spack` references, then use the `spack load [--sh|--csh|--fish]` capabilities to translate calls. The output is an self-modifying script that can then be run in the `spack` environment, to create the final, `unspacked` version of the original script.

```
$ unspack spacked.sh > unspacked.sh
$ bash unspacked.sh
```

The generated script can also be sourced, and will make the requested changes to the spack environment in addition to modifying itself.

# Disclaimer

- This will only work for `spack load` calls, and targets initialization scripts with little `spack` logic involved.
- The resulting scripts are also extremely single minded. They will overwrite whatever spack environment you have with what they were designed to do.
