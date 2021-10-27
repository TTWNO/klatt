#[cfg(test)]

use super::{db_to_lin, perform_frequency_modulation};

#[test]
fn db_to_lin_works() {
    assert_eq!(db_to_lin(-25f64), 0.05623413251903491);
    assert_eq!(db_to_lin(-20f64), 0.1);
    assert_eq!(db_to_lin(-15f64), 0.1778279410038923);
    assert_eq!(db_to_lin(-10f64), 0.31622776601683794);
    assert_eq!(db_to_lin(-5f64), 0.5623413251903491);
    assert_eq!(db_to_lin(0f64), 1.0);
}

#[test]
fn perform_frequency_modulation_works() {
    assert_eq!(perform_frequency_modulation(247.0, 0.25, 661.4476256042583), 247.8683597949849);
    assert_eq!(perform_frequency_modulation(247.0, 0.25, 661.4516618854374), 247.53177299547326);
    assert_eq!(perform_frequency_modulation(247.0, 0.25, 661.4556981666166), 247.1238618766483);
    assert_eq!(perform_frequency_modulation(247.0, 0.25, 661.4597344477957), 246.68625730966008);
    assert_eq!(perform_frequency_modulation(247.0, 0.25, 661.4637934047119), 246.2602657331645);
}