use std::io::Write;

fn main() {
    cargo_emit::rerun_if_changed!("src/ast/rhai.ungram");

    let ungram = std::fs::read_to_string("src/ast/rhai.ungram").unwrap();

    let generated = rhai_sourcegen::syntax::generate_syntax(&ungram).unwrap();

    let syntax_file = std::fs::read_to_string("src/syntax.rs").unwrap();

    let nodes_region_idx = syntax_file.find("// region: Nodes").unwrap();

    let nodes_region = &syntax_file[nodes_region_idx..];

    let nodes_region_end_idx = nodes_region.find("// endregion").unwrap();

    let new_syntax_file =
        String::from(&syntax_file[..nodes_region_idx + "// region: Nodes".len() + 1])
            + "    // This region is generated from ungrammar, do not touch it!\n"
            + &generated
                .node_kinds
                .into_iter()
                .map(|s| String::from("    ") + &s + ",\n")
                .collect::<String>()
            + "    "
            + &syntax_file[nodes_region_idx + nodes_region_end_idx..];

    let mut f = std::fs::File::create("src/syntax.rs").unwrap();

    f.write_all(new_syntax_file.as_bytes()).unwrap();

    let mut f = std::fs::File::create("src/ast/generated.rs").unwrap();
    f.write_all(generated.token_macro.as_bytes()).unwrap();
}
