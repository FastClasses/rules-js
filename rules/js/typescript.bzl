load(":node_modules.bzl", "create_node_modules_tree")
load(":providers.bzl", "JsLibraryInfo", "JsRuntimeInfo")

def _typescript_check_impl(ctx):
    npm_deps = [d for d in ctx.attrs.deps if JsLibraryInfo in d]
    node_modules = create_node_modules_tree(ctx, npm_deps)

    copy_map = {}

    for src in ctx.attrs.srcs:
        copy_map[src.short_path] = src

    copy_map[ctx.attrs.tsconfig.short_path] = ctx.attrs.tsconfig
    copy_map["node_modules"] = node_modules

    src_dir = ctx.actions.copied_dir("src_dir", copy_map)

    node_exe = ctx.attrs._js_runtime[JsRuntimeInfo].exe
    script = ctx.attrs._run_tsc

    stamp = ctx.actions.declare_output("tsc_stamp")

    cmd = cmd_args([node_exe, script, node_exe, src_dir, stamp.as_output()])

    ctx.actions.run(cmd, category = "typescript_check")

    return [DefaultInfo(default_output = stamp)]

typescript_check = rule(
    attrs = {
        "srcs": attrs.list(
            attrs.source(allow_directory = True),
            default = [],
        ),
        "tsconfig": attrs.source(default = "tsconfig.json"),
        "deps": attrs.list(attrs.dep()),
        "_js_runtime": attrs.dep(default = "toolchains//:js"),
        "_run_tsc": attrs.source(default = "//rules/js:run_tsc.mjs"),
    },
    impl = _typescript_check_impl,
)
