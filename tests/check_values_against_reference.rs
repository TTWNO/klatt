use hound::{
    WavReader,
};
use klatt::{FrameParms, GlottalSourceType, MainParms, generate_sound};
use rand::rngs::mock::StepRng;

/// When comparing against the reference sample, consider differences in value of:
/// 0.0000000001 to be acceptable.
///
/// Traditioanlly, this is called "epsilon".
const EPSILON: f32 = 1E-10;

fn m_parms() -> MainParms {
    MainParms {
        sample_rate: 44100,
        glottal_source_type: GlottalSourceType::Impulsive,
    }
}

fn f_params() -> FrameParms {
    FrameParms {
        duration: 1,
        f0: 247.0,
        flutter_level: 0.25,
        open_phase_ratio: 0.7,
        breathiness_db: -25.0,
        tilt_db: 0.0,
        gain_db: -10.0,
        agc_rms_level: 0.18,
        nasal_formant_freq: 1.0,
        nasal_formant_bw: 0.0,
        oral_formant_freq: vec![520.0, 1006.0, 2831.0, 3168.0, 4135.0, 5020.0],
        oral_formant_bw: vec![76.0, 102.0, 72.0, 102.0, 816.0, 596.0],
        cascade_enabled: true,
        cascade_voicing_db: 0.0,
        cascade_aspiration_db: -25.0,
        cascade_aspiration_mod: 0.5,
        nasal_antiformant_freq: 1.0,
        nasal_antiformant_bw: 0.0,
        parallel_enabled: true,
        parallel_voicing_db: 0.0,
        parallel_aspiration_db: -25.0,
        parallel_aspiration_mod: 0.5,
        frication_db: -30.0,
        frication_mod: 0.5,
        parallel_bypass_db: -99.0,
        nasal_formant_db: 0.0,
        oral_formant_db: vec![0.0, -8.0, -15.0, -19.0, -30.0, -35.0],
    }
}

#[test]
fn compare_to_reference_audio() {
    // used for deterministic, portable output
    let rng = StepRng::new(0, 0x12f6);
    let mut reader = WavReader::open(
        "reference.wav",
    ).unwrap();
    let sound = generate_sound(&m_parms(), &vec![f_params()], rng).unwrap();
    for (i,(maybe_ref_sample, gen_sample)) in reader.samples::<f32>().zip(sound.into_iter().map(|sample| sample as f32)).enumerate() {
        let Ok(ref_sample) = maybe_ref_sample else {
            panic!("The reference sample {i} is not able to be read from the wav file.");
        };
        assert!((ref_sample - gen_sample).abs() < EPSILON,
            "The generated sample is not within epsilon of the reference sample: abs({ref_sample} - {gen_sample}) > {EPSILON}");
    }
}
