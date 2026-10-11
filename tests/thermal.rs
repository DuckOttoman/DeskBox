use deskbox_reconstructed::thermal::{celsius,Thermal};
#[test] fn conversion_rejects_invalid(){assert!((celsius(301.).unwrap()-27.85).abs()<0.0001);for x in [0.,f64::NAN,f64::INFINITY,1000.]{assert_eq!(celsius(x),None)}}
#[test] #[ignore = "Requires firmware thermal counters on the test PC"] fn installed_thermal_counter(){let t=Thermal::open().expect("thermal counter on diagnostic computer");let v=t.sample().expect("valid thermal sample");println!("THERMAL_NATIVE_CELSIUS={v}");}
