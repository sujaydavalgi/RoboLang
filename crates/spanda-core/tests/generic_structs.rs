use spanda_core::check;

#[test]
fn generic_struct_type_params_type_check() {
    let source = r#"
struct Box<T> {
  value: T;
}

robot R {
  actuator wheels: DifferentialDrive;
}
"#;
    check(source).expect("generic struct declaration should type-check");
}
