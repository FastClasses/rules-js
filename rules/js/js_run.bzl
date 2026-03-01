load(":node_modules.bzl", "create_node_modules_tree")
load(":providers.bzl", "JsLibraryInfo", "JsRuntimeInfo")

def _js_run_impl(ctx):
    npm_deps = [d for d in ctx.attrs.deps if JsLibraryInfo in d]
    node_modules = create_node_modules_tree(ctx, npm_deps)

    copy_map = {}
    for src in ctx.attrs.srcs:
        copy_map[src.short_path] = src
    copy_map["node_modules"] = node_modules

    src_dir = ctx.actions.copied_dir("src_dir", copy_map)

    runtime = ctx.attrs.js_runtime[JsRuntimeInfo]
    exe = runtime.exe

    script = ctx.attrs._run_native_test
    entry = ctx.attrs.entry

    command = [exe]
    if runtime.runtime_name == "deno":
        command.extend(["run", "-A"])

    command.extend([script, exe, src_dir, entry] + ctx.attrs.run_args)

    return [
        DefaultInfo(),
        RunInfo(args = command),
    ]

js_run = rule(
    attrs = {
        "entry": attrs.string(),
        "run_args": attrs.list(
            attrs.string(),
            default = [],
        ),
        "srcs": attrs.list(
            attrs.source(allow_directory = True),
            default = [],
        ),
        "deps": attrs.list(
            attrs.dep(),
            default = [],
        ),
        "js_runtime": attrs.dep(default = "toolchains//:js"),
        "_run_native_test": attrs.source(default = "//rules/js:run_native_test.mjs"),
    },
    impl = _js_run_impl,
)
