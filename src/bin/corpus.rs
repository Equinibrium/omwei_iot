use omwei_iot::registry::CORPUS_V0_1;

fn main() {
    println!("OMWEI corpus v0.1");
    for id in [0x0042u16, 0x0043, 0x0044, 0x0045, 0x0046, 0x0047] {
        let d = CORPUS_V0_1.resolve(id).unwrap();
        println!("0x{:04X}  {:<24} {:<24} scale={}", d.id, d.label, d.unit, d.scale);
    }

    assert_eq!(CORPUS_V0_1.resolve(0x0042).unwrap().label, "ambient_temperature");
    assert_eq!(CORPUS_V0_1.resolve(0x0043).unwrap().label, "relative_humidity");
    assert_eq!(CORPUS_V0_1.resolve(0x0044).unwrap().label, "atmospheric_pressure");
    assert_eq!(CORPUS_V0_1.resolve(0x0045).unwrap().label, "active_power");
    assert_eq!(CORPUS_V0_1.resolve(0x0046).unwrap().label, "vibration_rms");
    assert_eq!(CORPUS_V0_1.resolve(0x0047).unwrap().label, "methane_concentration");
}
