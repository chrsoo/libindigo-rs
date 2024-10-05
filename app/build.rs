fn main() {
    glib_build_tools::compile_resources(
        &["libindigo-rs-example-app/resources"],
        "libindigo-rs-example-app/resources/resources.gresource.xml",
        "libindigo-rs-example-app.gresource",
    );
}