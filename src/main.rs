mod app_params;
mod klatt;
mod poly_real;

use klatt::{FrameParms, GlottalSourceType, MainParms};

fn run_generate_sound() {
    let sound = klatt::generate_sound(&app_params::m_parms(), &vec![app_params::f_params()]);
    match sound {
        Ok(sound) => println!("Sound: {:#?}", &sound[0..20]),
        Err(error) => {
            println!("Error: {}", error);
            std::process::exit(1);
        }
    }
}

fn run_generate_vocal() {
    let vocal = klatt::get_vocal_tract_transfer_function_coefficients(
        &app_params::m_parms(),
        &app_params::f_params(),
    );
    match vocal {
        Ok(vocal) => println!("Vocal: {:#?}", &vocal),
        Err(vocal) => {
            println!("Error: {}", vocal);
            std::process::exit(1);
        }
    }
}

fn main() {
    run_generate_sound();
    run_generate_vocal();
}
