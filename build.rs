fn main() {
    let audio_engine = "src/audio_engine.c";

    println!("cargo::rerun-if-changed={audio_engine}");

    cc::Build::new()
        .file(audio_engine)
        .flag_if_supported("-Wno-unused-parameter")
        .compile("audio_engine");

    println!("cargo::rustc-link-lib=framework=AudioUnit");
    println!("cargo::rustc-link-lib=framework=ApplicationServices");
}
