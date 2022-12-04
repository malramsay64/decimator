fn main() {
    glib_build_tools::compile_resources(
        "resources/",
        "resources/resources.gresources.xml",
        "decimator.gresource",
    )
}
