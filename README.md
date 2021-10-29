# klatt-syn-rs

## Launch
1. All the input parameters for the code are in `./src/app_p_arams.rs`
1. Open the project in VS Code and hit `F5`
1. Check resuls in the debug console

## Predictable results
To get the same results for every launch and make it possible to compare to TypeScripts's results, do the following changes to `.src/klatt.rs` file:
- in the method `get_white_noise()` replace 
``` Rust
return random::<f64>() * 2.0 - 1.0;
```
with
``` Rust
return 0.5;
```
- in the `new` function of the `Generator` class, replace
``` Rust
flutter_time_offset: (random::<f64>() * 1000.0) as usize,
```
with
``` Rust
flutter_time_offset: 555, 
```
