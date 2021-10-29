mod klatt;
mod poly_real;

use klatt::{FrameParms, GlottalSourceType, MainParms};

fn main() {
    run_generate_sound();
    run_generate_vocal();
}

fn get_m_parms() -> klatt::MainParms {
    MainParms {
        sample_rate: 44100,
        glottal_source_type: GlottalSourceType::Impulsive,
    }
}

fn get_f_params() -> FrameParms {
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

#[allow(dead_code)]
fn run_generate_sound() {
    let sound = klatt::generate_sound(&get_m_parms(), &vec![get_f_params()]);
    match sound {
        Ok(sound) => println!("Sound: {:#?}", &sound[0..20]),
        Err(error) => {
            println!("Error: {}", error);
            std::process::exit(1);
        }
    }
}

#[allow(dead_code)]
fn run_generate_vocal() {
    let vocal =
        klatt::get_vocal_tract_transfer_function_coefficients(&get_m_parms(), &get_f_params());
    match vocal {
        Ok(vocal) => println!("Vocal: {:#?}", &vocal),
        Err(vocal) => {
            println!("Error: {}", vocal);
            std::process::exit(1);
        }
    }
}
