use hassle_rs::intellisense::*;
use hassle_rs::*;

fn print_cursor_tree(cursor: &DxcCursor, source: &str) {
    print_indented_cursor_tree(cursor, source, 0)
}

fn print_indented_cursor_tree(cursor: &DxcCursor, source: &str, indent: usize) {
    print_indented_cursor(cursor, source, indent);

    let child_cursors = cursor.get_all_children().unwrap();

    for child_cursor in &child_cursors {
        print_indented_cursor_tree(child_cursor, source, indent + 1);
    }
}

fn print_indented_cursor(cursor: &DxcCursor, source: &str, indent: usize) {
    let range = cursor.get_extent().unwrap();
    let kind_flags = cursor.get_kind_flags().unwrap();
    let cursor_type = cursor.get_cursor_type().unwrap().get_spelling().unwrap();
    let spelling = cursor.get_spelling().unwrap();

    let display_name = cursor.get_display_name().unwrap();
    let format_name = cursor
        .get_formatted_name(DxcCursorFormatting::DEFAULT)
        .unwrap();
    let qualified_name = cursor.get_qualified_name(true).unwrap();

    let DxcSourceOffsets {
        start_offset,
        end_offset,
    } = range.get_offsets().unwrap();

    let source_range = (start_offset as usize)..(end_offset as usize);

    let source_text = &source[source_range];

    println!(
        "{: <indent$} - [{}:{}] {:?} kind_flags: {:?} cursor type: {:?} spelling: {:?}",
        "",
        start_offset,
        end_offset,
        display_name,
        kind_flags,
        cursor_type,
        spelling,
        indent = indent,
    );

    println!(
        "{: <indent$} > formatted name {:?}",
        "",
        format_name,
        indent = indent + 1
    );

    println!(
        "{: <indent$} > qualified name {:?}",
        "",
        qualified_name,
        indent = indent + 1
    );

    println!(
        "{: <indent$} > source {:?}",
        "",
        source_text,
        indent = indent + 1
    );
}

fn main() {
    let name = "copy.hlsl";

    let source = include_str!("copy.hlsl");

    let args = vec![];

    let dxc = Dxc::new(None).unwrap();

    let intellisense = dxc.create_intellisense().unwrap();

    let local_options = intellisense.get_default_editing_tu_options().unwrap();

    let index = intellisense.create_index().unwrap();

    let unsaved_file = intellisense.create_unsaved_file(name, source).unwrap();

    let translation_unit = index
        .parse_translation_unit(name, &args, &[&unsaved_file], local_options)
        .unwrap();

    let cursor = translation_unit.get_cursor().unwrap();

    print_cursor_tree(&cursor, source)
}
