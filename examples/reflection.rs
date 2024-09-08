use std::env::var;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source_copy = include_str!("copy.hlsl");
    let source_reflection = include_str!("reflection.hlsl");

    let dxc = hassle_rs::Dxc::new(None)?;

    let compiler = dxc.create_compiler()?;
    let library = dxc.create_library()?;

    let copy_blob = library.create_blob_with_encoding_from_str(source_copy)?;
    let reflection_blob = library.create_blob_with_encoding_from_str(source_reflection)?;

    println!("====== COMPUTE SHADER EXAMPLE ======");

    // Example with a compute shader
    match compiler.compile(
        &copy_blob,
        "copy.hlsl",
        "copyCs",
        "cs_6_0",
        &["-Od"], // "-Zi"
        Some(&mut hassle_rs::utils::DefaultIncludeHandler),
        &[],
    ) {
        Ok(result) => {
            let reflector = dxc.create_reflector()?;
            let reflected = reflector.reflect(result.get_result()?)?;

            let thread_group_size = reflected.thread_group_size();
            println!("thread group size {:?}", thread_group_size);

            let shader_desc = reflected.get_desc()?;
            println!("desc: {:?}", shader_desc);
        }
        Err((result, hresult)) => {
            println!("failed to compile shader");
            let error_blob = result.get_error_buffer()?;
            let error_string = library.get_blob_as_string(&error_blob.into())?;
            println!("{}", error_string);

        }
    }

    println!("====== VERTEX SHADER EXAMPLE ======");

    //
    match compiler.compile(
        &reflection_blob,
        "reflection.hlsl",
        "VSMain",
        "vs_6_0",
        &["-Od"], // "-Zi"
        Some(&mut hassle_rs::utils::DefaultIncludeHandler),
        &[],
    ) {
        Ok(result) => {
            let reflector = dxc.create_reflector()?;
            let reflected = reflector.reflect(result.get_result()?)?;

            let shader_desc = reflected.get_desc()?;
            println!("desc: {:?}", shader_desc);

            for i in 0..shader_desc.BoundResources {
                let resource_binding = reflected.get_resource_binding_desc(i)?;
                println!("resource {} {:?}", i, resource_binding);
            }

            for i in 0..shader_desc.InputParameters {
                let input_param = reflected.get_input_parameter_desc(i)?;
                println!("input param {} {:?}", i, input_param)
            }

            for i in 0..shader_desc.OutputParameters {
                let output_param = reflected.get_output_parameter_desc(i)?;
                println!("output param {} {:?}", i, output_param)
            }

            for i in 0..shader_desc.ConstantBuffers {
                println!("----- Constant Buffer {i}");
                let constant_buffer = reflected.get_constant_buffer_by_index(i);
                let constant_buffer_desc = constant_buffer.get_desc()?;
                println!("constant buffer name {:?}", constant_buffer_desc.Name);

                for i in 0..constant_buffer_desc.Variables {
                    let variable = constant_buffer.get_variable_by_index(i)?;
                    let variable_desc = variable.get_desc()?;
                    println!("var {}: {:?} +{} {:?}", i, variable_desc.Name, variable_desc.StartOffset, variable_desc);

                    let variable_type = variable.get_type();
                    let variable_type_desc = variable_type.get_desc()?;
                    println!("  type: {:?}", variable_type_desc);

                    for i in 0..variable_type_desc.Members {
                        let member_name = variable_type.get_member_type_name(i);
                        let member_desc = variable_type.get_desc();

                        println!("    member {} {:?} {:?}", i, member_name, member_desc);
                    }
                }
            }

            let cb = reflected.get_constant_buffer_by_name(c"SomeConstants").get_desc()?;
            println!("found by name {:?}", cb);

            let some_constants_type = reflected.get_constant_buffer_by_name(c"SomeConstants").get_variable_by_index(0)?.get_type();
            let some_constants_type_desc = some_constants_type.get_desc()?;


            println!("type name {:?} var count {:?}", some_constants_type_desc.Name, some_constants_type_desc.Members);
            for i in 0..some_constants_type_desc.Members {

                println!("member type name {:?}", some_constants_type.get_member_type_name(i));
                let member_type = some_constants_type.get_member_type_by_index(i);
                println!("member {:?}", member_type.get_desc());
            }
        }
        Err((result, hresult)) => {
            println!("failed to compile shader");
            let error_blob = result.get_error_buffer()?;
            let error_string = library.get_blob_as_string(&error_blob.into())?;
            println!("{}", error_string);

        }
    }

    Ok(())
}
