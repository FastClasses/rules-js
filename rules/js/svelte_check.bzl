load(":node_modules.bzl", "create_node_modules_tree")
load(":providers.bzl", "JsLibraryInfo", "JsRuntimeInfo")

def _svelte_check_impl(ctx):
    npm_deps = [d for d in ctx.attrs.deps if JsLibraryInfo in d]
    node_modules = create_node_modules_tree(ctx, npm_deps)

    copy_map = {}

    for src in ctx.attrs.srcs:
        copy_map[src.short_path] = src

    copy_map["node_modules"] = node_modules

    src_dir = ctx.actions.copied_dir("src_dir", copy_map)

    node_exe = ctx.attrs._js_runtime[JsRuntimeInfo].exe
    script = ctx.attrs._run_svelte_check

    return [
        DefaultInfo(),
        ExternalRunnerTestInfo(
            command = [node_exe, script, node_exe, src_dir],
            run_from_project_root = True,
            type = "svelte_check",
        ),
    ]

svelte_check_test = rule(
    attrs = {
        "srcs": attrs.list(
            attrs.source(allow_directory = True),
            default = [],
        ),
        "deps": attrs.list(attrs.dep()),
        "_js_runtime": attrs.dep(default = "toolchains//:js"),
        "_run_svelte_check": attrs.source(default = "//rules/js:run_svelte_check.mjs"),
    },
    impl = _svelte_check_impl,
)
