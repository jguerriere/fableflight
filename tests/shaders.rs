//! Parses and validates every shader without needing a GPU.
fn check(name: &str, src: &str) -> bool {
    match naga::front::wgsl::parse_str(src) {
        Err(e) => {
            eprintln!("--- {name}\n{}", e.emit_to_string(src));
            false
        }
        Ok(m) => match naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all()).validate(&m) {
            Err(e) => {
                eprintln!("--- {name}\n{}", e.emit_to_string(src));
                false
            }
            Ok(_) => true,
        },
    }
}

/// Every shader as render.rs assembles it (plane.wgsl takes the 747 deck's
/// string table: checked in b747::deck's tests).
fn sources() -> Vec<(&'static str, String)> {
    let common = include_str!("../src/shaders/common.wgsl");
    // Modules that draw the sky get sky.wgsl too, as in render.rs.
    let sky = include_str!("../src/shaders/sky.wgsl");
    let mut all: Vec<(&'static str, String)> = vec![
        ("noise3d", include_str!("../src/shaders/noise3d.wgsl").to_string()),
        ("resolve", include_str!("../src/shaders/resolve.wgsl").to_string()),
        ("detail", include_str!("../src/shaders/detail.wgsl").to_string()),
        ("ocean", include_str!("../src/shaders/ocean.wgsl").to_string()),
        ("shore", include_str!("../src/shaders/shore.wgsl").to_string()),
        ("csmix", include_str!("../src/shaders/csmix.wgsl").to_string()),
    ];
    for (name, src) in [
        ("lut", include_str!("../src/shaders/lut.wgsl")),
        ("terrain", include_str!("../src/shaders/terrain.wgsl")),
        ("photo", include_str!("../src/shaders/photo.wgsl")),
        ("post", include_str!("../src/shaders/post.wgsl")),
        ("particles_sim", include_str!("../src/shaders/particles_sim.wgsl")),
        ("particles_draw", include_str!("../src/shaders/particles_draw.wgsl")),
        ("roads", include_str!("../src/shaders/roads.wgsl")),
        ("trees", include_str!("../src/shaders/trees.wgsl")),
        ("vol", include_str!("../src/shaders/vol.wgsl")),
        ("ao", include_str!("../src/shaders/ao.wgsl")),
        ("taa", include_str!("../src/shaders/taa.wgsl")),
        ("airfield", include_str!("../src/shaders/airfield.wgsl")),
        ("lights", include_str!("../src/shaders/lights.wgsl")),
        ("precip", include_str!("../src/shaders/precip.wgsl")),
    ] {
        all.push((name, format!("{common}\n{src}")));
    }
    // As in render.rs, buildings and instanced solids share the facade shading.
    let facade = include_str!("../src/shaders/facade.wgsl");
    for (name, src) in [("buildings", include_str!("../src/shaders/buildings.wgsl")), ("inst", include_str!("../src/shaders/inst.wgsl"))] {
        all.push((name, format!("{common}\n{facade}\n{src}")));
    }
    for (name, src) in [("composite", include_str!("../src/shaders/composite.wgsl")), ("clouds", include_str!("../src/shaders/clouds.wgsl"))] {
        all.push((name, format!("{common}\n{sky}\n{src}")));
    }
    all
}

#[test]
fn shaders_validate() {
    let mut ok = true;
    for (name, src) in sources() {
        ok &= check(name, &src);
    }
    assert!(ok, "shader errors above");
}

/// On a Mac, the same shaders through naga's Metal writer and Apple's
/// compiler: what wgpu does when it builds the pipelines at startup.
#[cfg(target_os = "macos")]
#[path = "../src/shadercheck.rs"]
mod shadercheck;

#[cfg(target_os = "macos")]
#[test]
fn shaders_compile_for_metal() {
    let mut ok = true;
    for (name, src) in sources() {
        match shadercheck::compiles_for_metal(name, &src) {
            Ok(true) => {}
            Ok(false) => {
                eprintln!("no Metal device here: nothing compiled");
                return;
            }
            Err(e) => {
                eprintln!("{e}");
                ok = false;
            }
        }
    }
    assert!(ok, "shader errors above");
}
