macro_rules! compile_templates {
    ( $($name:literal),* $(,)?) => {{
        let mut env = Environment::new();

        $(
            env.add_template(
                $name,
                include_str!(concat!($name, ".j2")),
            )
            .expect(concat!("Unable to add template, ", $name));
        )*

        env
    }}
}
