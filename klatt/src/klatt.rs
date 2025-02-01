use crate::poly_real;
use alloc::{vec, vec::Vec};
use core::f64::consts::PI;
use core::prelude::rust_2024::derive;
use core::{
    cmp::PartialEq, option::Option, option::Option::None, option::Option::Some, result::Result,
    result::Result::Err, result::Result::Ok,
};
use libm::{cos, exp, pow, round, sin, sqrt};
use rand::Rng;

//--- Filters ------------------------------------------------------------------

/// A first-order IIR LP filter.
///
/// # Formulas:
/// ## Variables:
/// ```
///    x = input samples
///    y = output samples
///    a = first filter coefficient
///    b = second filter coefficient, >0 for LP filter, <0 for HP filter
///    f = frequency in Hz
///    w = 2 * PI * f / sampleRate
///    g = gain at frequency f
/// ```
/// ## Filter function:
/// ```
///    y[n] = a * x[n] + b * y[n-1]
/// ```
/// ## Transfer function:
/// ```
///    H(w) = a / ( 1 - b * e^(-jw) )
/// ```
/// ## Frequency response:
/// ```
///    |H(w)| = a / sqrt(1 - 2b * cos(w) + b^2)
/// ```
/// ## Gain at DC:
/// ```
///    |H(0)| = a / sqrt(1 - 2b * cos(0) + b^2)
///           = a / sqrt(1 - 2b + b^2)
///           = a / (1 - b)                                 for b < 1
/// ```
/// ## Cutoff frequency for LP filter (frequency with relative gain 0.5, about -3 dB):
/// ```
///    |H(fCutoff)| = |H(0)| / 2
///    a / sqrt(1 - 2b * cos(w) + b^2) = a / (2 * (1 - b))
///    fCutoff = acos((-3b^2 + 8b - 3) / 2b) * sampleRate / (2 * PI)
/// ```
/// ## Determine b for a given gain g at frequency f and |H(0)| = 1:
/// ```
///    a = 1 - b
///    g = (1 - b) / sqrt(1 - 2b * cos(w) + b^2)
///    g * sqrt(1 - 2b * cos(w) + b^2) = 1 - b
///    g^2 * (1 - 2b * cos(w) + b^2) = 1 - 2b + b^2
///    (g^2 - 1) * b^2  +  2 * (1 - g^2 * cos(w)) * b  +  g^2 - 1  =  0
///    b^2  +  2 * (1 - g^2 * cos(w)) / (g^2 - 1) * b  +  1  =  0
///    Substitute: q = (1 - g^2 * cos(w)) / (1 - g^2)
///    b^2 - 2 * q * b + 1 = 0
///    b = q - sqrt(q^2 - 1)                                or q + sqrt(q^2 - 1)
/// ```
struct LpFilter1 {
    sample_rate: usize,
    /// filter coefficient a
    a: f64,
    /// filter coefficient b
    b: f64,
    /// y[n-1], last output value
    y1: f64,
    passthrough: bool,
    muted: bool,
}
impl LpFilter1 {
    /// @param sampleRate
    ///    Sample rate in Hz.
    fn new(sample_rate: usize) -> Self {
        LpFilter1 {
            sample_rate,
            a: 0.0,
            b: 0.0,
            y1: 0.0,
            passthrough: true,
            muted: false,
        }
    }

    /// Adjusts the filter parameters without resetting the inner state.
    /// ### params
    /// ```
    ///    f = Frequency at which the gain is specified.
    ///    g = Gain at frequency f. Between 0 and 1 for LP filter. Greater than 1 for HP filter.
    ///    extra_gain = Extra gain factor. This is the resulting DC gain.
    /// ```
    /// The resulting gain at `f` will be `g * extraGain`.
    pub fn set(&mut self, f: f64, g: f64, extra_gain: Option<f64>) -> Result<(), &'static str> {
        let extra_gain = extra_gain.unwrap_or(1.0);
        if f <= 0.0
            || f >= self.sample_rate as f64 / 2.0
            || g <= 0.0
            || g >= 1.0
            || f.is_infinite()
            || g.is_infinite()
            || extra_gain.is_infinite()
        {
            return Err("Invalid filter parameters.");
        }

        let w = 2.0 * PI * f / (self.sample_rate as f64);
        let q = (1.0 - pow(g, 2.0) * cos(w)) / (1.0 - pow(g, 2.0));
        self.b = q - sqrt(pow(q, 2.0) - 1.0);
        self.a = (1.0 - self.b) * extra_gain;
        self.passthrough = false;
        self.muted = false;
        Ok(())
    }

    pub fn set_passthrough(&mut self) {
        self.passthrough = true;
        self.muted = false;
        self.y1 = 0.0;
    }

    #[allow(dead_code)]
    pub fn set_mute(&mut self) {
        self.passthrough = false;
        self.muted = true;
        self.y1 = 0.0;
    }

    /// Returns the polynomial coefficients of the filter transfer function in the z-plane.
    /// The returned array contains the top and bottom coefficients of the rational fraction, ordered in ascending powers.
    pub fn get_transfer_function_coefficients(&self) -> Vec<Vec<f64>> {
        if self.passthrough {
            return vec![vec![1.0], vec![1.0]];
        }
        if self.muted {
            return vec![vec![0.0], vec![1.0]];
        }
        vec![vec![self.a], vec![1.0, -self.b]]
    }

    /// Performs a filter step.
    /// ### params
    /// ```
    ///    x = Input signal value.
    /// ```
    /// ### returns
    ///    Output signal value.
    pub fn step(&mut self, x: f64) -> f64 {
        if self.passthrough {
            return x;
        }
        if self.muted {
            return 0.0;
        }
        let y = self.a * x + self.b * self.y1;
        self.y1 = y;
        y
    }
}

/// A Klatt resonator.
/// This is a second order IIR filter.
/// With f=0 it can also be used as a low-pass filter.
///
/// # Formulas:
/// ## Variables:
/// ```
///    x = input samples
///    y = output samples
///    a/b/c = filter coefficients
///    f = frequency in Hz
///    w = 2 * PI * f / sampleRate
///    f0 = resonator frequency in Hz
///    w0 = 2 * PI * f0 / sampleRate
///    bw = Bandwidth in Hz
///    r = exp(- PI * bw / sampleRate)
/// ```
/// ## Filter function:
/// ```
///    y[n] = a * x[n] + b * y[n-1] + c * y[n-2]
/// ```
/// ## Transfer function:
/// ```
///    H(w) = a / ( 1 - b * e^(-jw) - c * e^(-2jw) )
/// ```
/// ## Frequency response:
/// ```
///    |H(w)| = a / ( sqrt(1 + r^2 - 2 * r * cos(w - w0)) * sqrt(1 + r^2 - 2 * r * cos(w + w0)) )
/// ```
/// ## Gain at DC:
/// ```
///    |H(0)| = a / ( sqrt(1 + r^2 - 2 * r * cos(0 - w0)) * sqrt(1 + r^2 - 2 * r * cos(0 + w0)) )
///           = a / (1 + r^2 - 2 * r * cos(w0))
///           = a / (1 - c - b)
/// ```
/// ## Gain at the resonance frequency:
/// ```
///    |H(f0)| = a / sqrt(1 + r^2 - 2 * r)
///            = a / (1 - r)
/// ```
struct Resonator {
    sample_rate: usize,
    /// filter coefficient a
    a: f64,
    /// filter coefficient b
    b: f64,
    /// filter coefficient c
    c: f64,
    /// y[n-1], last output value
    y1: f64,
    /// y[n-2], second-last output value
    y2: f64,
    r: f64,
    passthrough: bool,
    muted: bool,
}
impl Resonator {
    /// ### params
    /// ```
    /// sample_rate = Sample rate in Hz.
    /// ```
    fn new(sample_rate: usize) -> Self {
        Resonator {
            sample_rate,
            a: 0.0,
            b: 0.0,
            c: 0.0,
            y1: 0.0,
            y2: 0.0,
            r: 0.0,
            passthrough: true,
            muted: false,
        }
    }
    /// Adjusts the filter parameters without resetting the inner state.
    /// ### params
    /// ```
    /// f = Frequency of resonator in Hz. May be 0 for LP filtering.
    /// bw = Bandwidth of resonator in Hz.
    /// dc_gain = DC gain level.
    /// ```
    pub fn set(&mut self, f: f64, bw: f64, dc_gain: Option<f64>) -> Result<(), &'static str> {
        let dc_gain = dc_gain.unwrap_or(1.0);
        if f < 0.0
            || f >= self.sample_rate as f64 / 2.0
            || bw <= 0.0
            || dc_gain <= 0.0
            || f.is_infinite()
            || bw.is_infinite()
            || dc_gain.is_infinite()
        {
            return Err("Invalid resonator parameters.");
        }
        self.r = exp(-PI * bw / (self.sample_rate as f64));
        let w = 2.0 * PI * f / (self.sample_rate as f64);
        self.c = -pow(self.r, 2.0);
        self.b = 2.0 * self.r * cos(w);
        self.a = (1.0 - self.b - self.c) * dc_gain;
        self.passthrough = false;
        self.muted = false;
        Ok(())
    }

    pub fn set_passthrough(&mut self) {
        self.passthrough = true;
        self.muted = false;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }

    pub fn set_mute(&mut self) {
        self.passthrough = false;
        self.muted = true;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }

    pub fn adjust_impulse_gain(&mut self, new_a: f64) {
        self.a = new_a;
    }

    pub fn adjust_peak_gain(&mut self, peak_gain: f64) -> Result<(), &'static str> {
        if peak_gain <= 0.0 || peak_gain.is_infinite() {
            return Err("Invalid resonator peak gain.");
        }
        self.a = peak_gain * (1.0 - self.r);
        Ok(())
    }

    /// Returns the polynomial coefficients of the filter transfer function in the z-plane.
    /// The returned array contains the top and bottom coefficients of the rational fraction, ordered in ascending powers.
    pub fn get_transfer_function_coefficients(&self) -> Vec<Vec<f64>> {
        if self.passthrough {
            return vec![vec![1.0], vec![1.0]];
        }
        if self.muted {
            return vec![vec![0.0], vec![1.0]];
        }
        vec![vec![self.a], vec![1.0, -self.b, -self.c]]
    }

    /// Performs a filter step.
    /// ### params
    /// ```
    ///    x = Input signal value.
    /// ```
    /// ### returns
    ///    Output signal value.
    pub fn step(&mut self, x: f64) -> f64 {
        if self.passthrough {
            return x;
        }
        if self.muted {
            return 0.0;
        }
        let y = self.a * x + self.b * self.y1 + self.c * self.y2;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

/// A Klatt anti-resonator.
/// This is a second order FIR filter.
///
/// # Formulas:
/// ## Variables:
/// ```
///    x = input samples
///    y = output samples
///    a/b/c = filter coefficients
///    f = frequency in Hz
///    w = 2 * PI * f / sampleRate
/// ```
/// # Filter function:
/// ```
///    y[n] = a * x[n] + b * x[n-1] + c * x[n-2]
/// ```
/// # Transfer function:
/// ```
///    H(w) = a + b * e^(-jw) + c * e^(-2jw)
/// ```
struct AntiResonator {
    sample_rate: usize,
    /// filter coefficient a
    a: f64,
    /// filter coefficient b
    b: f64,
    /// filter coefficient c
    c: f64,
    /// x[n-1], last input value
    x1: f64,
    /// x[n-2], second-last input value
    x2: f64,
    passthrough: bool,
    muted: bool,
}
impl AntiResonator {
    /// ### params
    /// ```
    ///    sample_rate = Sample rate in Hz.
    /// ```
    pub fn new(sample_rate: usize) -> Self {
        AntiResonator {
            sample_rate,

            a: 0.0,
            b: 0.0,
            c: 0.0,
            x1: 0.0,
            x2: 0.0,
            passthrough: true,
            muted: false,
        }
    }

    /// Adjusts the filter parameters without resetting the inner state.
    /// ### params
    /// ```
    ///    f = Frequency of anti-resonator in Hz.
    ///    bw = bandwidth of anti-resonator in Hz.
    /// ```
    pub fn set(&mut self, f: f64, bw: f64) -> Result<(), &'static str> {
        if f <= 0.0
            || f >= self.sample_rate as f64 / 2.0
            || bw <= 0.0
            || f.is_infinite()
            || bw.is_infinite()
        {
            return Err("Invalid anti-resonator parameters.");
        }
        let r = exp(-PI * bw / (self.sample_rate as f64));
        let w = 2.0 * PI * f / (self.sample_rate as f64);
        let c0 = -(r * r);
        let b0 = 2.0 * r * cos(w);
        let a0 = 1.0 - b0 - c0;
        if a0 == 0.0 {
            self.a = 0.0;
            self.b = 0.0;
            self.c = 0.0;
            return Ok(());
        }
        self.a = 1.0 / a0;
        self.b = -b0 / a0;
        self.c = -c0 / a0;
        self.passthrough = false;
        self.muted = false;
        Ok(())
    }

    pub fn set_passthrough(&mut self) {
        self.passthrough = true;
        self.muted = false;
        self.x1 = 0.0;
        self.x2 = 0.0;
    }

    #[allow(dead_code)]
    pub fn set_mute(&mut self) {
        self.passthrough = false;
        self.muted = true;
        self.x1 = 0.0;
        self.x2 = 0.0;
    }

    /// Returns the polynomial coefficients of the filter transfer function in the z-plane.
    /// The returned array contains the top and bottom coefficients of the rational fraction, ordered in ascending powers.
    pub fn get_transfer_function_coefficients(&self) -> Vec<Vec<f64>> {
        if self.passthrough {
            return vec![vec![1.0], vec![1.0]];
        }
        if self.muted {
            return vec![vec![0.0], vec![1.0]];
        }
        vec![vec![self.a, self.b, self.c], vec![1.0]]
    }
    /// Performs a filter step.
    /// ### params
    /// ```
    ///    x = Input signal value.
    /// ```
    /// ### returns
    ///    Output signal value.
    pub fn step(&mut self, x: f64) -> f64 {
        if self.passthrough {
            return x;
        }
        if self.muted {
            return 0.0;
        }
        let y = self.a * x + self.b * self.x1 + self.c * self.x2;
        self.x2 = self.x1;
        self.x1 = x;
        y
    }
}

/// A differencing filter.
/// This is a first-order FIR HP filter.
///
/// # Problem:
/// The filter curve depends on the sample rate.
/// # TODO:
/// Compensate the effect of the sample rate.
///
/// # Formulas:
/// ## Variables:
/// ```
///    x = input samples
///    y = output samples
///    f = frequency in Hz
///    w = 2 * PI * f / sampleRate
/// ```
/// ## Filter function:
/// ```
///    y[n] = x[n] - x[n-1]
/// ```
/// ## Transfer function:
/// ```
///    H(w) = 1 - e^(-jw)
/// ```
/// ## Frequency response:
/// ```
///    |H(w)| = sqrt(2 - 2 * cos(w))
/// ```
struct DifferencingFilter {
    /// x[n-1], last input value
    x1: f64,
}
impl DifferencingFilter {
    pub fn new() -> Self {
        DifferencingFilter { x1: 0.0 }
    }
    // Returns the polynomial coefficients of the filter transfer function in the z-plane.
    // The returned array contains the top and bottom coefficients of the rational fraction, ordered in ascending powers.
    pub fn get_transfer_function_coefficients(&self) -> Vec<Vec<f64>> {
        vec![vec![1.0, -1.0], vec![1.0]]
    }
    /// Performs a filter step.
    /// ### params
    /// ```
    ///    x = Input signal value.
    /// ```
    /// ### returns
    ///    Output signal value.
    pub fn step(&mut self, x: f64) -> f64 {
        let y = x - self.x1;
        self.x1 = x;
        y
    }
}

//--- Noise sources ------------------------------------------------------------

/// Returns a random number within the range -1 .. 1.
fn get_white_noise<R: Rng>(rng: &mut R) -> f64 {
    let x = rng.random_range(-1.0..=1.0);
    assert!(x >= -1.0, "{x} is too small");
    assert!(x <= 1.0, "{x} is too big");
    x
}

/// A low-pass filtered noise source.
struct LpNoiseSource<R> {
    lp_filter: LpFilter1,
    rng: R,
}
impl<R: Rng> LpNoiseSource<R> {
    pub fn new(sample_rate: usize, rng: R) -> Result<Self, &'static str> {
        // The original program logic used a first order LP filter with a filter coefficient
        // of b=0.75 and a sample rate of 10 kHz.
        let old_b = 0.75;
        let old_ample_rate = 10000.0;
        // Compute the gain at 1000 Hz with a sample rate of 10 kHz and a DC gain of 1.
        let f = 1000.0;
        let g = (1.0 - old_b)
            / sqrt(1.0 - 2.0 * old_b * cos(2.0 * PI * f / old_ample_rate) + pow(old_b, 2.0));

        // compensate amplitude for output range -1 .. +1
        // Create an LP filter with the same characteristics but with our sampling rate.
        let extra_gain = 2.5 * pow(sample_rate as f64 / 10000_f64, 0.33);

        let mut lp_noise_source = LpNoiseSource {
            lp_filter: LpFilter1::new(sample_rate),
            rng,
        };
        lp_noise_source.lp_filter.set(f, g, Some(extra_gain))?;
        Ok(lp_noise_source)
    }

    /// Returns an LP-filtered random number.
    pub fn get_next(&mut self) -> f64 {
        let x = get_white_noise(&mut self.rng);
        self.lp_filter.step(x)
    }
}

//--- Glottal sources ----------------------------------------------------------

/// Generates a glottal source signal by LP filtering a pulse train.
struct ImpulsiveGlottalSource {
    sample_rate: usize,
    /// resonator used as an LP filter
    resonator: Option<Resonator>,
    /// current sample position within F0 period
    position_in_period: usize,
}
impl ImpulsiveGlottalSource {
    pub fn new(sample_rate: usize) -> Self {
        ImpulsiveGlottalSource {
            sample_rate,
            resonator: None,
            position_in_period: 0,
        }
    }
    /// ### params
    /// ```
    ///    open_phase_length = Duration of the open glottis phase of the F0 period, in samples.
    /// ```
    pub fn start_period(&mut self, open_phase_length: usize) -> Result<(), &'static str> {
        if open_phase_length == 0 {
            self.resonator = None;
            return Ok(());
        }
        if self.resonator.is_none() {
            self.resonator = Some(Resonator::new(self.sample_rate));
        }
        let bw = (self.sample_rate as f64) / (open_phase_length as f64);
        self.resonator.as_mut().unwrap().set(0.0, bw, None)?;
        self.resonator.as_mut().unwrap().adjust_impulse_gain(1.0);
        self.position_in_period = 0;

        Ok(())
    }

    pub fn get_next(&mut self) -> f64 {
        if self.resonator.is_none() {
            return 0.0;
        }

        let pulse = if self.position_in_period == 1 {
            1.0
        } else if self.position_in_period == 2 {
            -1.0
        } else {
            0.0
        };

        self.position_in_period += 1;
        self.resonator.as_mut().unwrap().step(pulse)
    }
}

/// Generates a "natural" glottal source signal according to the KLGLOTT88 model.
/// Formula of the glottal flow: `t^2 - t^3`
/// Formula of the derivative: `2 * t - 3 * t^2`
/// The derivative is used as the glottal source.
///
/// At the end of the open glottal phase there is an abrupt jump from the minimum value to zero.
/// This jump is not smoothed in the classic Klatt model. In Praat this "collision phase" is smoothed.
struct NaturalGlottalSource {
    /// current signal value
    x: f64,
    /// current first derivative
    a: f64,
    /// current second derivative
    b: f64,
    /// open glottis phase length in samples
    open_phase_length: usize,
    /// current sample position within F0 period
    position_in_period: usize,
}
impl NaturalGlottalSource {
    pub fn new() -> Self {
        let mut natural_glottal_source = NaturalGlottalSource {
            x: 0.0,
            a: 0.0,
            b: 0.0,
            open_phase_length: 0,
            position_in_period: 0,
        };

        natural_glottal_source.start_period(0);
        natural_glottal_source
    }

    /// ### params
    /// ```
    ///    open_phase_length = Duration of the open glottis phase of the F0 period, in samples.
    /// ```
    pub fn start_period(&mut self, open_phase_length: usize) {
        self.open_phase_length = open_phase_length;
        self.x = 0.0;
        let amplification = 5.0;
        self.b = -amplification / pow(open_phase_length as f64, 2.0);
        self.a = -self.b * open_phase_length as f64 / 3.0;
        self.position_in_period = 0;
    }

    pub fn get_next(&mut self) -> f64 {
        self.position_in_period += 1;
        if self.position_in_period >= self.open_phase_length {
            self.x = 0.0;
            return 0.0;
        }
        self.a += self.b;
        self.x += self.a;
        self.x
    }
}

//------------------------------------------------------------------------------

/// Modulates the fundamental frequency (F0).
///
/// Sine-wave frequencies of 12.7, 7.1 and 4.7 Hz were chosen so as to ensure
/// a long period before repetition of the perturbation that is introduced.
/// A value of flutterLevel = 0.25 results in synthetic vowels with a quite
/// realistic deviation from constant pitch.
///
/// ### params
/// ```
///    f0 = Fundamental frequency.
///    flutter_level = Flutter level between 0 and 1.
///    time = Relative signal position in seconds.
/// ```
/// ### returns
///    Modulated fundamental frequency.
fn perform_frequency_modulation(f0: f64, flutter_level: f64, time: f64) -> f64 {
    if flutter_level <= 0.0 {
        return f0;
    }
    let w = 2.0 * PI * time;
    let a = sin(12.7 * w) + sin(7.1 * w) + sin(4.7 * w);

    f0 * (1.0 + a * flutter_level / 50.0)
}

/// Convert a dB value into a linear value.
/// dB values of -99 and below or NaN are converted to 0.
fn db_to_lin(db: f64) -> f64 {
    if db <= -99.0 || db.is_nan() {
        0.0
    } else {
        pow(10.0, db / 20.0)
    }
}

//--- Main logic ---------------------------------------------------------------

pub enum GlottalSourceType {
    Impulsive,
    Natural,
    Noise,
}

pub const MAX_ORAL_FORMANTS: usize = 6;

/// Parameters for the whole sound.
pub struct MainParms {
    /// sample rate in Hz
    pub sample_rate: usize,
    pub glottal_source_type: GlottalSourceType,
}

/// Parameters for a sound frame.
#[derive(PartialEq)]
pub struct FrameParms {
    /// frame duration in seconds
    pub duration: usize,
    /// fundamental frequency in Hz
    pub f0: f64,
    /// F0 flutter level, 0 .. 1, typically 0.25
    pub flutter_level: f64,
    /// relative length of the open phase of the glottis, 0 .. 1, typically 0.7
    pub open_phase_ratio: f64,
    /// breathiness in voicing (turbulence) in dB, positive to amplify or negative to attenuate
    pub breathiness_db: f64,
    /// spectral tilt for glottal source in dB. Attenuation at 3 kHz in dB. 0 = no tilt.
    pub tilt_db: f64,
    /// overall gain (output gain) in dB, positive to amplify, negative to attenuate, NaN for automatic gain control (AGC)
    pub gain_db: f64,
    /// RMS level for automatic gain control (AGC), only relevant when gainDb is NaN
    pub agc_rms_level: f64,
    /// nasal formant frequency in Hz, or NaN
    pub nasal_formant_freq: f64,
    /// nasal formant bandwidth in Hz, or NaN
    pub nasal_formant_bw: f64,
    /// oral format frequencies in Hz, or NaN
    pub oral_formant_freq: Vec<f64>,
    /// oral format bandwidths in Hz, or NaN
    pub oral_formant_bw: Vec<f64>,

    // Cascade branch:
    /// true = cascade branch enabled
    pub cascade_enabled: bool,
    /// voicing amplitude for cascade branch in dB, positive to amplify or negative to attenuate
    pub cascade_voicing_db: f64,
    /// aspiration (glottis noise) amplitude for cascade branch in dB, positive to amplify or negative to attenuate
    pub cascade_aspiration_db: f64,
    /// amplitude modulation factor for aspiration in cascade branch, 0 = no modulation, 1 = maximum modulation
    pub cascade_aspiration_mod: f64,
    /// nasal antiformant frequency in Hz, or NaN
    pub nasal_antiformant_freq: f64,
    /// nasal antiformant bandwidth in Hz, or NaN
    pub nasal_antiformant_bw: f64,

    // Parallel branch:
    /// true = parallel branch enabled
    pub parallel_enabled: bool,
    /// voicing amplitude for parallel branch in dB, positive to amplify or negative to attenuate
    pub parallel_voicing_db: f64,
    /// aspiration (glottis noise) amplitude for parallel branch in dB, positive to amplify or negative to attenuate
    pub parallel_aspiration_db: f64,
    /// amplitude modulation factor for aspiration in parallel branch, 0 = no modulation, 1 = maximum modulation
    pub parallel_aspiration_mod: f64,
    /// frication noise level in dB
    pub frication_db: f64,
    /// amplitude modulation factor for frication noise in parallel branch, 0 = no modulation, 1 = maximum modulation
    pub frication_mod: f64,
    /// parallel bypass level in dB, used to bypass differentiated glottal and frication signals around resonators F2 to F6
    pub parallel_bypass_db: f64,
    /// nasal formant level in dB
    pub nasal_formant_db: f64,
    /// oral format levels in dB, or NaN
    pub oral_formant_db: Vec<f64>,
}

/// Variables of the currently active frame.
#[allow(clippy::struct_field_names)]
struct FrameState {
    /// linear breathiness level
    pub breathiness_lin: f64,
    /// linear overall gain
    pub gain_lin: f64,

    // Cascade branch:
    /// linear voicing amplitude for cascade branch
    pub cascade_voicing_lin: f64,
    /// linear aspiration amplitude for cascade branch
    pub cascade_aspiration_lin: f64,

    // Parallel branch:
    /// linear voicing amplitude for parallel branch
    parallel_voicing_lin: f64,
    /// linear aspiration amplitude for parallel branch
    parallel_aspiration_lin: f64,
    /// linear frication noise level
    frication_lin: f64,
    /// linear parallel bypass level
    parallel_bypass_lin: f64,
}
impl FrameState {
    pub fn new() -> Self {
        FrameState {
            breathiness_lin: 0.0,
            gain_lin: 0.0,
            cascade_voicing_lin: 0.0,
            cascade_aspiration_lin: 0.0,
            parallel_voicing_lin: 0.0,
            parallel_aspiration_lin: 0.0,
            frication_lin: 0.0,
            parallel_bypass_lin: 0.0,
        }
    }
}

/// Variables of the currently active F0 period (aka glottal period).
/// F0 period state
struct PeriodState {
    /// modulated fundamental frequency for this period, in Hz, or 0
    pub f0: f64,
    /// period length in samples
    period_length: usize,
    /// open glottis phase length in samples
    pub open_phase_length: usize,

    // Per sample values:
    /// current sample position within F0 period
    pub position_in_period: usize,
    /// LP filtered noise
    #[allow(dead_code)]
    lp_noise: usize,
}
impl PeriodState {
    pub fn new() -> Self {
        PeriodState {
            f0: 0.0,
            period_length: 0,
            open_phase_length: 0,
            position_in_period: 0,
            lp_noise: 0,
        }
    }
}

/// Sound generator controller.
pub struct Generator<'a, R> {
    /// main parameters
    m_parms: &'a MainParms,
    /// currently active frame parameters
    f_parms: Option<&'a FrameParms>,
    /// new frame parameters for start of next F0 period
    new_f_parms: Option<&'a FrameParms>,
    /// frame variables
    f_state: FrameState,
    /// F0 period state variables
    p_state: Option<PeriodState>,
    /// current absolute sample position
    abs_position: usize,
    /// spectral tilt filter
    tilt_filter: LpFilter1,
    /// output low-pass filter
    output_lp_filter: Resonator,
    /// random value for flutter time offset
    flutter_time_offset: usize,

    // Glottal source:
    impulsive_g_source: Option<ImpulsiveGlottalSource>,
    natural_g_source: Option<NaturalGlottalSource>,
    /// function which returns the next glottal source signal sample value
    glottal_source: fn(&mut Generator<R>) -> f64,

    // Noise sources:
    // (We use independent noise sources to avoid cancellation effects of correlated signals.)
    /// noise source for aspiration in cascade branch
    aspiration_source_casc: LpNoiseSource<R>,
    /// noise source for aspiration in parallel branch
    aspiration_source_par: LpNoiseSource<R>,
    /// noise source for frication in parallel branch
    frication_source_par: LpNoiseSource<R>,

    // Cascade branch variables:
    /// nasal formant filter for cascade branch
    nasal_formant_casc: Resonator,
    /// nasal antiformant filter for cascade branch
    nasal_antiformant_casc: AntiResonator,
    /// oral formant filters for cascade branch
    oral_formant_casc: Vec<Resonator>,

    // Parallel branch variables:
    /// nasal formant filter for parallel branch
    nasal_formant_par: Resonator,
    /// oral formant filters for parallel branch
    oral_formant_par: Vec<Resonator>,
    /// differencing filter for the parallel branch
    differencing_filter_par: DifferencingFilter,
    /// random number generator function
    rng: R,
}
impl<'a, R: Rng + Clone> Generator<'a, R> {
    pub fn new(m_parms: &MainParms, mut rng: R) -> Result<Generator<R>, &'static str> {
        let mut generator = Generator {
            m_parms,
            f_state: FrameState::new(),
            abs_position: 0,
            tilt_filter: LpFilter1::new(m_parms.sample_rate),
            flutter_time_offset: rng.random_range(0..=1000),
            output_lp_filter: Resonator::new(m_parms.sample_rate),
            f_parms: None,
            new_f_parms: None,
            p_state: None,

            // Glottal source:
            impulsive_g_source: None,
            natural_g_source: None,
            glottal_source: |_g: &mut Generator<R>| 0.0,

            // Create noise sources:
            aspiration_source_casc: LpNoiseSource::new(m_parms.sample_rate, rng.clone())?,
            aspiration_source_par: LpNoiseSource::new(m_parms.sample_rate, rng.clone())?,
            frication_source_par: LpNoiseSource::new(m_parms.sample_rate, rng.clone())?,

            // Initialize cascade branch variables:
            nasal_formant_casc: Resonator::new(m_parms.sample_rate),
            nasal_antiformant_casc: AntiResonator::new(m_parms.sample_rate),
            oral_formant_casc: Vec::with_capacity(MAX_ORAL_FORMANTS),

            // Initialize parallel branch variables:
            nasal_formant_par: Resonator::new(m_parms.sample_rate),
            oral_formant_par: Vec::with_capacity(MAX_ORAL_FORMANTS),
            differencing_filter_par: DifferencingFilter::new(),
            rng,
        };

        generator
            .output_lp_filter
            .set(0.0, (m_parms.sample_rate as f64) / 2.0, None)?;

        generator.init_glottal_source();

        for _ in 0..MAX_ORAL_FORMANTS {
            generator
                .oral_formant_casc
                .push(Resonator::new(m_parms.sample_rate));
            generator
                .oral_formant_par
                .push(Resonator::new(m_parms.sample_rate));
        }

        Ok(generator)
    }

    /// Generates a frame of the sound.
    /// The length of the frame is specified by `outBuf.length` and `fParms.duration` is ignored.
    pub fn generate_frame(
        &mut self,
        f_parms: &'a FrameParms,
        out_buf: &mut [f64],
    ) -> Result<(), &'static str> {
        if let Some(parms) = self.f_parms {
            if parms == f_parms {
                return Err("FrameParms structure must not be re-used.");
            }
        }

        self.new_f_parms = Some(f_parms);
        for out_pos in &mut *out_buf {
            match &self.p_state {
                Some(p_state) => {
                    if p_state.position_in_period >= p_state.period_length {
                        self.start_new_period()?;
                    }
                }
                None => self.start_new_period()?,
            }

            *out_pos = self.compute_next_output_signal_sample();
            self.p_state.as_mut().unwrap().position_in_period += 1;
            self.abs_position += 1;
        }

        // automatic gain control (AGC)
        if f_parms.gain_db.is_nan() {
            adjust_signal_gain(out_buf, f_parms.agc_rms_level);
        }

        Ok(())
    }

    fn compute_next_output_signal_sample(&mut self) -> f64 {
        let glottan_source: fn(&mut Generator<R>) -> f64 = self.glottal_source;
        let mut voice = glottan_source(self);

        let f_parms = self.f_parms.unwrap();
        let p_state = self.p_state.as_ref().unwrap();

        // apply spectral tilt
        voice = self.tilt_filter.step(voice);

        // if within glottal open phase
        if p_state.position_in_period < p_state.open_phase_length {
            // add breathiness (turbulence)
            voice += get_white_noise(&mut self.rng) * self.f_state.breathiness_lin;
        }

        let cascade_out = if f_parms.cascade_enabled {
            self.compute_cascade_branch(voice)
        } else {
            0.0
        };

        let parallel_out = if f_parms.parallel_enabled {
            self.compute_parallel_branch(voice)
        } else {
            0.0
        };

        let mut out = cascade_out + parallel_out;
        out = self.output_lp_filter.step(out);
        out *= self.f_state.gain_lin;
        out
    }

    fn compute_cascade_branch(&mut self, voice: f64) -> f64 {
        let f_parms = self.f_parms.unwrap();
        let p_state = self.p_state.as_ref().unwrap();
        let cascade_voice = voice * self.f_state.cascade_voicing_lin;

        let current_aspiration_mod = if p_state.position_in_period >= p_state.period_length / 2 {
            f_parms.cascade_aspiration_mod
        } else {
            0.0
        };

        let aspiration = self.aspiration_source_casc.get_next()
            * self.f_state.cascade_aspiration_lin
            * (1.0 - current_aspiration_mod);
        let mut v = cascade_voice + aspiration;
        v = self.nasal_antiformant_casc.step(v);
        v = self.nasal_formant_casc.step(v);
        for i in 0..MAX_ORAL_FORMANTS {
            v = self.oral_formant_casc[i].step(v);
        }
        v
    }

    fn compute_parallel_branch(&mut self, voice: f64) -> f64 {
        let f_parms = self.f_parms.unwrap();
        let p_state = self.p_state.as_ref().unwrap();
        let parallel_voice = voice * self.f_state.parallel_voicing_lin;

        let current_aspiration_mod = if p_state.position_in_period >= p_state.period_length / 2 {
            f_parms.parallel_aspiration_mod
        } else {
            0.0
        };

        let aspiration = self.aspiration_source_par.get_next()
            * self.f_state.parallel_aspiration_lin
            * (1.0 - current_aspiration_mod);
        let source = parallel_voice + aspiration;
        let source_difference = self.differencing_filter_par.step(source);
        // Klatt (1980) states: "... using a first difference calculation to remove low-frequency energy from
        // the higher formants; this energy would otherwise distort the spectrum in the region of F1 during
        // the synthesis of some vowels."
        // A differencing filter is applied for H2 to H6 and the bypass.
        // A better solution would probably be to use real band-pass filters instead of resonators for the formants
        // in the parallel branch. Then this differencing filter would not be necessary to protect the low frequencies
        // of the low formants.
        let current_frication_mod = if p_state.position_in_period >= p_state.period_length / 2 {
            f_parms.frication_mod
        } else {
            0.0
        };

        let frication_noise = self.frication_source_par.get_next()
            * self.f_state.frication_lin
            * (1.0 - current_frication_mod);
        let source2 = source_difference + frication_noise;
        let mut v = 0.0;
        v += self.nasal_formant_par.step(source); // nasal formant is directly applied to source
        v += self.oral_formant_par[0].step(source); // F1 is directly applied to source
        for i in 1..MAX_ORAL_FORMANTS {
            // F2 to F6 are applied to source difference + frication
            let alternating_sign = if i % 2 == 0 { 1.0 } else { -1.0 }; // (refer to Klatt (1980) Fig. 13)
            v += alternating_sign * self.oral_formant_par[i].step(source2);
        }
        // bypass is applied to source difference + frication
        v += self.f_state.parallel_bypass_lin * source2;
        v
    }

    /// Starts a new F0 period.
    // this is fine because it only operates on two variables:
    //
    // - period_length
    // - sample_rate
    //
    // Both of which will do.... something weird if it ends up being negative.
    #[allow(clippy::cast_sign_loss)]
    fn start_new_period(&mut self) -> Result<(), &'static str> {
        if let Some(new_f_parms) = self.new_f_parms {
            // To reduce glitches, new frame parameters are only activated at the start of a new F0 period.
            self.f_parms = Some(new_f_parms);
            self.new_f_parms = None;
            self.start_using_new_frame_parameters()?;
        }
        if self.p_state.is_none() {
            self.p_state = Some(PeriodState::new());
        }
        let p_state = self.p_state.as_mut().unwrap();
        let f_parms = self.f_parms.unwrap();
        let flutter_time = self.abs_position / self.m_parms.sample_rate + self.flutter_time_offset;
        p_state.f0 =
            perform_frequency_modulation(f_parms.f0, f_parms.flutter_level, flutter_time as f64);

        p_state.period_length = if p_state.f0 > 0.0 {
            round((self.m_parms.sample_rate as f64) / p_state.f0) as usize
        } else {
            1
        };

        p_state.open_phase_length = if p_state.period_length > 1 {
            round((p_state.period_length as f64) * f_parms.open_phase_ratio) as usize
        } else {
            0
        };

        p_state.position_in_period = 0;
        self.start_glottal_source_period()?;
        Ok(())
    }

    fn start_using_new_frame_parameters(&mut self) -> Result<(), &'static str> {
        let f_parms = self.f_parms.unwrap();
        self.f_state.breathiness_lin = db_to_lin(f_parms.breathiness_db);
        self.f_state.gain_lin = db_to_lin(f_parms.gain_db);
        let db = if f_parms.gain_db.is_finite() {
            f_parms.gain_db
        } else {
            0.0
        };
        self.f_state.gain_lin = db_to_lin(db);
        set_tilt_filter(&mut self.tilt_filter, f_parms.tilt_db)?;

        // Adjust cascade branch:
        self.f_state.cascade_voicing_lin = db_to_lin(f_parms.cascade_voicing_db);
        self.f_state.cascade_aspiration_lin = db_to_lin(f_parms.cascade_aspiration_db);
        set_nasal_formant_casc(&mut self.nasal_formant_casc, f_parms)?;
        set_nasal_antiformant_casc(&mut self.nasal_antiformant_casc, f_parms)?;
        for i in 0..MAX_ORAL_FORMANTS {
            set_oral_formant_casc(&mut self.oral_formant_casc[i], f_parms, i)?;
        }

        // Adjust parallel branch:
        self.f_state.parallel_voicing_lin = db_to_lin(f_parms.parallel_voicing_db);
        self.f_state.parallel_aspiration_lin = db_to_lin(f_parms.parallel_aspiration_db);
        self.f_state.frication_lin = db_to_lin(f_parms.frication_db);
        self.f_state.parallel_bypass_lin = db_to_lin(f_parms.parallel_bypass_db);
        set_nasal_formant_par(&mut self.nasal_formant_par, f_parms)?;
        for i in 0..MAX_ORAL_FORMANTS {
            set_oral_formant_par(&mut self.oral_formant_par[i], self.m_parms, f_parms, i)?;
        }
        Ok(())
    }

    fn init_glottal_source(&mut self) {
        match self.m_parms.glottal_source_type {
            GlottalSourceType::Impulsive => {
                self.impulsive_g_source =
                    Some(ImpulsiveGlottalSource::new(self.m_parms.sample_rate));
                self.glottal_source =
                    |g: &mut Generator<R>| g.impulsive_g_source.as_mut().unwrap().get_next();
            }
            GlottalSourceType::Natural => {
                self.natural_g_source = Some(NaturalGlottalSource::new());
                self.glottal_source =
                    |g: &mut Generator<R>| g.natural_g_source.as_mut().unwrap().get_next();
            }
            GlottalSourceType::Noise => {
                self.glottal_source = |g: &mut Generator<R>| get_white_noise(&mut g.rng);
            }
        }
    }

    fn start_glottal_source_period(&mut self) -> Result<(), &'static str> {
        match self.m_parms.glottal_source_type {
            GlottalSourceType::Impulsive => self
                .impulsive_g_source
                .as_mut()
                .unwrap()
                .start_period(self.p_state.as_ref().unwrap().open_phase_length),
            GlottalSourceType::Natural => {
                self.natural_g_source
                    .as_mut()
                    .unwrap()
                    .start_period(self.p_state.as_ref().unwrap().open_phase_length);
                Ok(())
            }
            GlottalSourceType::Noise => Ok(()),
        }
    }
}

fn set_tilt_filter(tilt_filter: &mut LpFilter1, tilt_db: f64) -> Result<(), &'static str> {
    if tilt_db == 0.0 {
        tilt_filter.set_passthrough();
    } else {
        tilt_filter.set(3000.0, db_to_lin(-tilt_db), None)?;
    }
    Ok(())
}

fn set_nasal_formant_casc(
    nasal_formant_casc: &mut Resonator,
    f_parms: &FrameParms,
) -> Result<(), &'static str> {
    if f_parms.nasal_formant_freq != 0.0 && f_parms.nasal_formant_bw != 0.0 {
        nasal_formant_casc.set(f_parms.nasal_formant_freq, f_parms.nasal_formant_bw, None)?;
    } else {
        nasal_formant_casc.set_passthrough();
    }
    Ok(())
}

fn set_nasal_antiformant_casc(
    nasal_antiformant_casc: &mut AntiResonator,
    f_parms: &FrameParms,
) -> Result<(), &'static str> {
    if f_parms.nasal_antiformant_freq != 0.0 && f_parms.nasal_antiformant_bw != 0.0 {
        nasal_antiformant_casc.set(f_parms.nasal_antiformant_freq, f_parms.nasal_antiformant_bw)?;
    } else {
        nasal_antiformant_casc.set_passthrough();
    }
    Ok(())
}

fn set_oral_formant_casc(
    oral_formant_casc: &mut Resonator,
    f_parms: &FrameParms,
    i: usize,
) -> Result<(), &'static str> {
    let f = if i < f_parms.oral_formant_freq.len() {
        f_parms.oral_formant_freq[i]
    } else {
        f64::NAN
    };

    let bw = if i < f_parms.oral_formant_bw.len() {
        f_parms.oral_formant_bw[i]
    } else {
        f64::NAN
    };

    if f.is_finite() && bw.is_finite() {
        oral_formant_casc.set(f, bw, None)?;
    } else {
        oral_formant_casc.set_passthrough();
    }
    Ok(())
}

fn set_nasal_formant_par(
    nasal_formant_par: &mut Resonator,
    f_parms: &FrameParms,
) -> Result<(), &'static str> {
    if f_parms.nasal_formant_freq != 0.0
        && f_parms.nasal_formant_bw != 0.0
        && db_to_lin(f_parms.nasal_formant_db) != 0.0
    {
        nasal_formant_par.set(f_parms.nasal_formant_freq, f_parms.nasal_formant_bw, None)?;
        nasal_formant_par.adjust_peak_gain(db_to_lin(f_parms.nasal_formant_db))?;
    } else {
        nasal_formant_par.set_mute();
    }
    Ok(())
}

fn set_oral_formant_par(
    oral_formant_par: &mut Resonator,
    m_parms: &MainParms,
    f_parms: &FrameParms,
    i: usize,
) -> Result<(), &'static str> {
    let formant = i + 1;
    let f = if i < f_parms.oral_formant_freq.len() {
        f_parms.oral_formant_freq[i]
    } else {
        f64::NAN
    };

    let bw = if i < f_parms.oral_formant_bw.len() {
        f_parms.oral_formant_bw[i]
    } else {
        f64::NAN
    };

    let db = if i < f_parms.oral_formant_db.len() {
        f_parms.oral_formant_db[i]
    } else {
        f64::NAN
    };

    let peak_gain = db_to_lin(db);
    // Klatt used the following linear factors to adjust the levels of the parallel formant
    // resonators so that they have a similar effect as the cascade versions:
    //   F1: 0.4, F2: 0.15, F3: 0.06, F4: 0.04, F5: 0.022, F6: 0.03, Nasal: 0.6
    // We are not doing this here, because then the output of the parallel branch would no longer
    // match the specified formant levels. Instead, we use the specified dB value to set the peak gain
    // instead of taking it as the DC gain.
    if f.is_finite() && bw.is_finite() && peak_gain.is_finite() {
        oral_formant_par.set(f, bw, None)?;
        let w = 2.0 * PI * f / (m_parms.sample_rate as f64);
        let diff_gain = sqrt(2.0 - 2.0 * cos(w)); // gain of differencing filter

        // compensate differencing filter for F2 to F6
        let filter_gain = if formant >= 2 {
            peak_gain / diff_gain
        } else {
            peak_gain
        };

        oral_formant_par.adjust_peak_gain(filter_gain)?;
    } else {
        oral_formant_par.set_mute();
    }
    Ok(())
}

fn adjust_signal_gain(buf: &mut [f64], target_rms: f64) {
    let n = buf.len();
    if n == 0 {
        return;
    }
    // let rms = 21f64;
    let rms = compute_rms(buf);
    if rms == 0.0 {
        return;
    }
    let r = target_rms / rms;
    for b_i in buf.iter_mut() {
        *b_i *= r;
    }
}

fn compute_rms(buf: &[f64]) -> f64 {
    sqrt(buf.iter().map(|f| pow(*f, 2.0)).sum::<f64>() / buf.len() as f64)
}

//------------------------------------------------------------------------------

/// Generates a sound that consists of multiple frames.
///
/// # Errors
///
/// Returns a static str if there is a problem with the [`m_parms`] and `f_parms_a`] values.
pub fn generate_sound<R: Rng + Clone>(
    m_parms: &MainParms,
    f_parms_a: &Vec<FrameParms>,
    rng: R,
) -> Result<Vec<f64>, &'static str> {
    let mut generator = Generator::new(m_parms, rng)?;
    let mut out_buf_len = 0;
    for f_parms in f_parms_a {
        out_buf_len += f_parms.duration * m_parms.sample_rate;
    }
    let mut out_buf: Vec<f64> = vec![0.0; out_buf_len];

    let mut out_buf_pos = 0;
    for f_parms in f_parms_a {
        let frame_len = f_parms.duration * m_parms.sample_rate;
        let frame_buf = &mut out_buf[out_buf_pos..(out_buf_pos + frame_len)];
        generator.generate_frame(f_parms, frame_buf)?;
        out_buf_pos += frame_len;
    }
    Ok(out_buf)
}

//--- Transfer function --------------------------------------------------------

const EPS: f64 = 1E-10;

/// Returns the polynomial coefficients of the overall filter transfer function in the z-plane.
/// The returned array contains the top and bottom coefficients of the rational fraction, ordered in ascending powers.
///
/// # Errors
///
/// Any invalid parameters will return a static str explaining the invalid param.
pub fn get_vocal_tract_transfer_function_coefficients(
    m_parms: &MainParms,
    f_parms: &FrameParms,
) -> Result<Vec<Vec<f64>>, &'static str> {
    // glottal source
    let mut voice: Vec<Vec<f64>> = vec![vec![1.0], vec![1.0]];
    //
    let mut tilt_filter = LpFilter1::new(m_parms.sample_rate);
    set_tilt_filter(&mut tilt_filter, f_parms.tilt_db)?;
    let tilt_trans = &tilt_filter.get_transfer_function_coefficients();
    voice = poly_real::multiply_fractions(&voice, tilt_trans, Some(EPS))?;
    //
    let cascade_trans = if f_parms.cascade_enabled {
        get_cascade_branch_transfer_function_coefficients(m_parms, f_parms)?
    } else {
        vec![vec![0.0], vec![1.0]]
    };
    let parallel_trans = if f_parms.parallel_enabled {
        get_parallel_branch_transfer_function_coefficients(m_parms, f_parms)?
    } else {
        vec![vec![0.0], vec![1.0]]
    };
    let branches_trans = poly_real::add_fractions(&cascade_trans, &parallel_trans, Some(EPS))?;
    let mut out = poly_real::multiply_fractions(&voice, &branches_trans, Some(EPS))?;
    //
    let mut output_lp_filter = Resonator::new(m_parms.sample_rate);
    output_lp_filter.set(0.0, m_parms.sample_rate as f64 / 2.0, None)?;
    let output_lp_trans = output_lp_filter.get_transfer_function_coefficients();
    out = poly_real::multiply_fractions(&out, &output_lp_trans, Some(EPS))?;
    //
    let db = if f_parms.gain_db.is_finite() {
        f_parms.gain_db
    } else {
        0.0
    };
    let gain_lin = db_to_lin(db);
    out = poly_real::multiply_fractions(&out, &[vec![gain_lin], vec![1.0]], Some(EPS))?;
    //
    Ok(out)
}

fn get_cascade_branch_transfer_function_coefficients(
    m_parms: &MainParms,
    f_parms: &FrameParms,
) -> Result<Vec<Vec<f64>>, &'static str> {
    let cascade_voicing_lin = db_to_lin(f_parms.cascade_voicing_db);
    let mut v: Vec<Vec<f64>> = vec![vec![cascade_voicing_lin], vec![1.0]];
    //
    let mut nasal_antiformant_casc = AntiResonator::new(m_parms.sample_rate);
    set_nasal_antiformant_casc(&mut nasal_antiformant_casc, f_parms)?;
    let nasal_antiformant_trans = nasal_antiformant_casc.get_transfer_function_coefficients();
    v = poly_real::multiply_fractions(&v, &nasal_antiformant_trans, Some(EPS))?;
    //
    let mut nasal_formant_casc = Resonator::new(m_parms.sample_rate);
    set_nasal_formant_casc(&mut nasal_formant_casc, f_parms)?;
    let nasal_formant_trans = nasal_formant_casc.get_transfer_function_coefficients();
    v = poly_real::multiply_fractions(&v, &nasal_formant_trans, Some(EPS))?;
    //
    for i in 0..MAX_ORAL_FORMANTS {
        let mut oral_formant_casc = Resonator::new(m_parms.sample_rate);
        set_oral_formant_casc(&mut oral_formant_casc, f_parms, i)?;
        let oral_formant_casc_trans = oral_formant_casc.get_transfer_function_coefficients();
        v = poly_real::multiply_fractions(&v, &oral_formant_casc_trans, Some(EPS))?;
    }
    //
    Ok(v)
}

fn get_parallel_branch_transfer_function_coefficients(
    m_parms: &MainParms,
    f_parms: &FrameParms,
) -> Result<Vec<Vec<f64>>, &'static str> {
    let parallel_voicing_lin = db_to_lin(f_parms.parallel_voicing_db);
    let source: Vec<Vec<f64>> = vec![vec![parallel_voicing_lin], vec![1.0]];
    //
    let differencing_filter_par = DifferencingFilter::new();
    let differencing_filter_trans = differencing_filter_par.get_transfer_function_coefficients();
    let source2 = poly_real::multiply_fractions(&source, &differencing_filter_trans, Some(EPS))?;
    //
    let mut v: Vec<Vec<f64>> = vec![vec![0.0], vec![1.0]];
    //
    let mut nasal_formant_par = Resonator::new(m_parms.sample_rate);
    set_nasal_formant_par(&mut nasal_formant_par, f_parms)?;
    let nasal_formant_trans = nasal_formant_par.get_transfer_function_coefficients();
    v = poly_real::add_fractions(
        &v,
        &poly_real::multiply_fractions(&source, &nasal_formant_trans, None)?,
        Some(EPS),
    )?;
    //
    for i in 0..MAX_ORAL_FORMANTS {
        let mut oral_formant_par = Resonator::new(m_parms.sample_rate);
        set_oral_formant_par(&mut oral_formant_par, m_parms, f_parms, i)?;
        let oral_pformant_trans = oral_formant_par.get_transfer_function_coefficients();
        // F1 is applied to source, F2 to F6 are applied to difference
        let formant_in = if i == 0 { &source } else { &source2 };
        let formant_out =
            poly_real::multiply_fractions(formant_in, &oral_pformant_trans, Some(EPS))?;
        let alternating_sign = if i % 2 == 0 { 1.0 } else { -1.0 };
        let v2 = poly_real::multiply_fractions(
            &formant_out,
            &[vec![alternating_sign], vec![1.0]],
            Some(EPS),
        )?;
        v = poly_real::add_fractions(&v, &v2, Some(EPS))?;
    }
    //
    let parallel_bypass_lin = db_to_lin(f_parms.parallel_bypass_db);
    // bypass is applied to source difference
    let parallel_bypass = poly_real::multiply_fractions(
        &source2,
        &[vec![parallel_bypass_lin], vec![1.0]],
        Some(EPS),
    )?;
    v = poly_real::add_fractions(&v, &parallel_bypass, Some(EPS))?;
    //
    Ok(v)
}
