load(":providers.bzl", "JsLibraryInfo")

def _js_library_impl(ctx):
    direct_deps = [d[JsLibraryInfo] for d in ctx.attrs.deps if JsLibraryInfo in d]

    all_deps_map = {}
    for d in direct_deps:
        all_deps_map[d.package_name] = d
        if hasattr(d, "deps"):
            for td in d.deps:
                all_deps_map[td.package_name] = td

    transitive_deps = all_deps_map.values()

    mapping = {}

    target_dir = ctx.attrs.name + "/"
    for src in ctx.attrs.srcs:
        short_path = src.short_path
        if target_dir in short_path:
            mapping[short_path.split(target_dir, 1)[1]] = src
        else:
            mapping[src.basename] = src

    out_dir = ctx.actions.copied_dir(ctx.attrs.name, mapping)

    return [
        DefaultInfo(default_output = out_dir),
        JsLibraryInfo(
            package_name = ctx.attrs.package_name,
            out_dir = out_dir,
            version = ctx.attrs.version,
            deps = transitive_deps,
        ),
    ]

js_library = rule(
    attrs = {
        "package_name": attrs.string(default = ""),
        "version": attrs.string(default = ""),
        "srcs": attrs.list(
            attrs.source(),
            default = [],
        ),
        "deps": attrs.list(
            attrs.dep(),
            default = [],
        ),
    },
    impl = _js_library_impl,
)
