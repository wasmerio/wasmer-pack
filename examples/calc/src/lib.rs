wai_bindgen_rust::export!("calc.export.wai");

struct Calc;

impl calc::Calc for Calc {
    fn add(a: f32, b: f32) -> f32 {
        a + b
    }
}
