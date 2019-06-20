use hassle_rs::*;

fn main() {
    let name = "copy.hlsl";

    let source = include_str!("copy.hlsl");

    let args = vec![];

    let dxc = Dxc::new();

    let intellisense = dxc.create_intellisense().unwrap();

    let local_options = intellisense.get_default_editing_tu_options().unwrap();

    let index = intellisense.create_index().unwrap();

    let unsaved_file = intellisense
        .create_unsaved_file(name.as_bytes(), source.as_bytes())
        .unwrap();

    let translation_unit = index
        .parse_translation_unit(name.as_bytes(), &args, &vec![&unsaved_file], local_options)
        .unwrap();

    let cursor = translation_unit.get_cursor().unwrap();

    let result = intellisense::utils::print_cursor_tree(&cursor);
}
