mod app_params;
#[allow(dead_code)]
mod klatt;
mod poly_real;
use hound::{
	WavWriter,
	WavSpec,
	SampleFormat,
};

fn run_generate_sound() {
		let mut wav = WavWriter::create("out.wav", WavSpec {
				channels: 1,
				sample_rate: 44100,
				bits_per_sample: 32,
				sample_format: SampleFormat::Float,
		}).unwrap();
    let sound = klatt::generate_sound(&app_params::m_parms(), &vec![app_params::f_params()]);
    match sound {
        Ok(sound) => {
					for sample in sound {
            println!("{:?}", sample);
						let s2: f32 = sample as f32;
						wav.write_sample(s2).unwrap();
					}
				},
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
