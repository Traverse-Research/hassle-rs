#![allow(clippy::uninlined_format_args)]

use hassle_rs::*;

fn main() {
    let name = "copy.hlsl";

    let source = include_str!("copy.hlsl");

    let args = vec![];

    let dxc = unsafe { Dxc::linked_or_load() }.unwrap();

    let intellisense = dxc.create_intellisense().unwrap();

    let local_options = intellisense.get_default_editing_tu_options().unwrap();

    let index = intellisense.create_index().unwrap();

    let unsaved_file = intellisense.create_unsaved_file(name, source).unwrap();

    let translation_unit = index
        .parse_translation_unit(name, &args, &[&unsaved_file], local_options)
        .unwrap();

    let cursor = translation_unit.get_cursor().unwrap();

    {
        let range = cursor.get_extent().unwrap();
        println!("Range {:?}", range);

        let location = cursor.get_location().unwrap();
        println!("Location {:?}", location);

        let name = cursor.get_display_name().unwrap();
        println!("Name {:?}", name);
        assert_eq!(name, "copy.hlsl");

        let cursor_kind = cursor.get_kind().unwrap();
        println!("CursorKind {:?}", cursor_kind);

        let cursor_kind_flags = cursor.get_kind_flags().unwrap();
        println!("CursorKindFlags {:?}", cursor_kind_flags);
    }

    let child_cursors = cursor.get_all_children().unwrap();

    assert_eq!(child_cursors[0].get_display_name().unwrap(), "g_input");
    assert_eq!(child_cursors[1].get_display_name().unwrap(), "g_output");
    assert_eq!(
        child_cursors[2].get_display_name().unwrap(),
        "copyCs(uint3)"
    );

    for child_cursor in child_cursors {
        let range = child_cursor.get_extent().unwrap();
        println!("Child Range {:?}", range);

        let location = child_cursor.get_location().unwrap();
        println!("Child Location {:?}", location);

        let name = child_cursor.get_display_name().unwrap();
        println!("Child Name {:?}", name);

        let cursor_kind = child_cursor.get_kind().unwrap();
        println!("Child CursorKind {:?}", cursor_kind);

        let cursor_kind_flags = child_cursor.get_kind_flags().unwrap();
        println!("CursorKindFlags {:?}", cursor_kind_flags);
    }
}
