load(":node_modules.bzl", "create_node_modules_tree")
load(":providers.bzl", "JsLibraryInfo", "JsRuntimeInfo")

def _vite_build_impl(ctx):
    npm_deps = [d for d in ctx.attrs.deps if JsLibraryInfo in d]
    node_modules = create_node_modules_tree(ctx, npm_deps)

    copy_map = {}

    for src in ctx.attrs.srcs:
        copy_map[src.short_path] = src

    copy_map[ctx.attrs.vite_config.short_path] = ctx.attrs.vite_config
    copy_map["node_modules"] = node_modules

    src_dir = ctx.actions.copied_dir("src_dir", copy_map)
    out_build = ctx.actions.declare_output(ctx.attrs.out_dir)

    node_exe = ctx.attrs._js_runtime[JsRuntimeInfo].exe
    vite_js = cmd_args(src_dir, format = "{}/node_modules/vite/bin/vite.js")

    script = ctx.attrs._build_vite
    cmd = cmd_args([node_exe, script, src_dir, node_exe, vite_js, ctx.attrs.out_dir, out_build.as_output()])

    ctx.actions.run(
        cmd,
        category = "vite_build",
    )

    return [DefaultInfo(default_output = out_build)]

vite_build = rule(
    attrs = {
        "srcs": attrs.list(
            attrs.source(allow_directory = True),
            default = [],
        ),
        "vite_config": attrs.source(),
        "deps": attrs.list(attrs.dep()),
        "out_dir": attrs.string(default = "dist"),
        "_js_runtime": attrs.dep(default = "toolchains//:js"),
        "_build_vite": attrs.source(default = "//rules/js:build_vite.mjs"),
    },
    impl = _vite_build_impl,
)
