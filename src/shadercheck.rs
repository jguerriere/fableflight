//! Test helper: compiles a WGSL shader the way the game does on a Mac.
//!
//! naga's validator accepts WGSL that its Metal writer turns into source
//! Apple's compiler then refuses, and wgpu only finds out when it builds the
//! pipeline, at startup, on a Mac. This translates each entry point to MSL as
//! wgpu-hal does and hands it to the system's Metal compiler, so such a shader
//! fails a test instead. Shared by tests/shaders.rs and the unit tests
//! (`#[path]`), since plane.wgsl needs the binary's 747 string table.
#![cfg(target_os = "macos")]

use objc2_foundation::NSString;
use objc2_metal::{MTLCompileOptions, MTLCreateSystemDefaultDevice, MTLDevice};

/// Compiles every entry point of `src` for Metal. `Ok(false)` when this Mac
/// has no Metal device to compile with (nothing was checked).
pub fn compiles_for_metal(name: &str, src: &str) -> Result<bool, String> {
    let Some(device) = MTLCreateSystemDefaultDevice() else { return Ok(false) };
    let module = naga::front::wgsl::parse_str(src).map_err(|e| format!("--- {name}\n{}", e.emit_to_string(src)))?;
    let info = naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all()).validate(&module).map_err(|e| format!("--- {name}\n{}", e.emit_to_string(src)))?;
    // (the bindings come from the pipeline layout in the game; any slot will do to compile)
    let options = naga::back::msl::Options { lang_version: (2, 4), fake_missing_bindings: true, ..Default::default() };
    for ep in &module.entry_points {
        let pipeline = naga::back::msl::PipelineOptions { entry_point: Some((ep.stage, ep.name.clone())), ..Default::default() };
        let (msl, _) = naga::back::msl::write_string(&module, &info, &options, &pipeline).map_err(|e| format!("--- {name} ({}): naga could not write MSL: {e:?}", ep.name))?;
        let compile = MTLCompileOptions::new();
        #[allow(unused_unsafe)]
        let library = unsafe { device.newLibraryWithSource_options_error(&NSString::from_str(&msl), Some(&compile)) };
        if let Err(e) = library {
            // (the first lines hold the error; the rest lists every overload Metal tried)
            let text: String = e.localizedDescription().to_string().lines().filter(|l| !l.contains("warning:")).take(12).collect::<Vec<_>>().join("\n");
            return Err(format!("--- {name} ({}): Metal refuses the translated shader\n{text}", ep.name));
        }
    }
    Ok(true)
}
