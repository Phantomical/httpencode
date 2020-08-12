const HAS_CONST_PANIC: &str = r#"
{
  const DUMMY: () = panic!("Panic message");
}
"#;

fn main() {
  let cfg = autocfg::new();

  cfg.emit_expression_cfg(HAS_CONST_PANIC, "has_const_panic");
}
