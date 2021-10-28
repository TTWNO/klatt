mod lib;
use lib::{FrameParms, GlottalSourceType, MainParms};
use rand::prelude::*;

fn main() {
    // println!("Hello, world!");

    // let mut rng = rand::thread_rng();

    // let n1: u8 = rng.gen();
    // let n2: u16 = rng.gen();
    // println!("Random u8: {}", n1);
    // println!("Random u16: {}", n2);
    // println!("Random u32: {}", rng.gen::<u32>());
    // println!("Random i32: {}", rng.gen::<i32>());
    // println!("Random float: {}", rng.gen::<f64>());
    // println!("Random range(0..1000): {}", rng.gen_range(0..999));

    // let val = lib::perform_frequency_modulation(247f64, 0.25, 661.4476256042583);
    // println!("{}", val);

    // println!("{}", lib::dbToLin(-25f64));
    // println!("{}", lib::dbToLin(-15f64));
    // println!("{}", lib::dbToLin(-10f64));
    // println!("{}", lib::dbToLin(-5f64));
    // println!("{}", lib::dbToLin(0f64));

    // run_random_generator();

    run_generate_sound();
}

#[allow(dead_code)]
fn run_random_generator() {
    let mut rng = StdRng::seed_from_u64(32);
    let mut result = Vec::new();

    for _i in 0..10 {
        let value: f64 = rng.gen();
        result.push(value);
    }

    println!("{:#?}", result);
}

#[allow(dead_code)]
fn run_generate_sound() {
    let m_parms = MainParms {
        sample_rate: 44100,
        glottal_source_type: GlottalSourceType::Impulsive,
    };

    let f_params = FrameParms {
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
        parallel_enabled: false,
        parallel_voicing_db: 0.0,
        parallel_aspiration_db: -25.0,
        parallel_aspiration_mod: 0.5,
        frication_db: -30.0,
        frication_mod: 0.5,
        parallel_bypass_db: -99.0,
        nasal_formant_db: 0.0,
        oral_formant_db: vec![0.0, -8.0, -15.0, -19.0, -30.0, -35.0],
    };

    let f_parms_a = vec![f_params];

    let sound = lib::generate_sound(&m_parms, &f_parms_a);

    match sound {
        Ok(sound) => println!("Sound: {:#?}", &sound[0..10]),
        Err(error) => {
            println!("Error: {}", error);
            std::process::exit(1);
        }
    }
}
