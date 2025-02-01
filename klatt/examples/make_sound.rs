use rand::{
	rngs::SmallRng,
	SeedableRng,
};
use klatt::{
	generate_sound,
	get_vocal_tract_transfer_function_coefficients,
	m_parms,
	f_params,
};
use hound::{
	WavWriter,
	WavSpec,
	SampleFormat,
};

fn run_generate_sound() {
		let rng = SmallRng::seed_from_u64(69);
		let mut wav = WavWriter::create("out.wav", WavSpec {
				channels: 1,
				sample_rate: 16000,
				bits_per_sample: 32,
				sample_format: SampleFormat::Float,
		}).unwrap();
    let sound = generate_sound(&m_parms(), &vec![f_params()], rng);
    match sound {
        Ok(sound) => {
					for sample in sound {
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
    let vocal = get_vocal_tract_transfer_function_coefficients(
        &m_parms(),
        &f_params(),
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
