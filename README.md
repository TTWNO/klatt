# `klatt`

Klatt Formant Speech Synthesis in Rust.
This is based on the original reference implementation included in the `reference-implementation/` directory.

Made to be fully `no_std` compatible.

**NOTE:** This is _not a text to speech engine_!
This is a set of algorithms designed to synthesize "speech sounding" audio samples from a variety of voice-related parameters.
Text-to-speech engines can use this to generate speech, but they need to have the right kind of parameters to feed into this algorithm.

## Future Goals

- [ ] `no_alloc`
    - Useful in specialized embedded environments.
- [ ] async integration
    - For example, to asynchronously produce a frame of audio every N samples.
- [ ] More examples
    - Currently only one example.
    - Exapand to using alternate parameters and concatenation to produce basic speech sounds.
- [ ] Integration into a TTS engine
    - The long-term goal is to create a `no_std` compatible text-to-speech engine.

## Predictable results

To generate predictable results, use the `StepRng` struct as defined in the `examples/make_sound.rs`.
This allows you to test against changes to make sure it didn't break anything :)

## `no_std` Support

This library is `no_std` compatible by disabling default features, and enabling the `libm` feature;
this allows math operations not included in `core`.
We also take a dependency on `rand`; make sure if you use it, you disable its `std`-dependent features.

THe primary way to make sound—through the `generate_sound` function—is generic over any `Rng` implementation.
Feel free to write your own, or use `SmallRng` for use in embedded environments.

