# klatt-syn-rs

This is a conversion from TypeScript to Rust of [KlattSyn - Klatt Formant Synthesizer](https://github.com/chdh/klatt-syn)
## Launch

1. Open the project in VS Code
1.Hit `F5` and see result in the terminal

All the input parameters for the code are in `./src/app_params.rs`

## Predictable results
To get the same results for every launch and make it possible to compare to TypeScript's results you need to block the random number generator with a fixed value. To do that do the following changes to `.src/klatt.rs` file:
- in the method `get_white_noise()` replace 
``` Rust
return rand::random::<f64>() * 2.0 - 1.0;
```
with
``` Rust
return 0.5;
```
- in the `new` function of the `Generator` class, replace
``` Rust
flutter_time_offset: (rand::random::<f64>() * 1000.0) as usize,
```
with
``` Rust
flutter_time_offset: 555, 
```

Do the corresponding changes to the TypeScript sources cloned from [github.com/chdh/klatt-syn](https://github.com/chdh/klatt-syn)

In the `.src/Klatt.ts` file:
- in the method `getWhiteNoise()` replace 
``` TypeScript
return Math.random() * 2 - 1;
```
with
``` TypeScript
return 0.5
```
- in the `constructor` of the `Generator` class, replace
``` TypeScript
this.flutterTimeOffset = Math.random() * 1000;
```
with
``` TypeScript
this.flutterTimeOffset = 555;
```
