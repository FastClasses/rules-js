load(":providers.bzl", "JsLibraryInfo", "JsRuntimeInfo")
load(":node_modules.bzl", "create_node_modules_tree")

def _vitest_test_impl(ctx):
    npm_deps = [d for d in ctx.attrs.deps if JsLibraryInfo in d]
    node_modules = create_node_modules_tree(ctx, npm_deps)

    copy_map = {}
    
    for src in ctx.attrs.srcs:
        copy_map[src.short_path] = src
        
    if ctx.attrs.vitest_config:
        copy_map[ctx.attrs.vitest_config.short_path] = ctx.attrs.vitest_config
        
    copy_map["node_modules"] = node_modules

    src_dir = ctx.actions.copied_dir("src_dir", copy_map)

    node_exe = ctx.attrs._js_runtime[JsRuntimeInfo].exe
    script = ctx.attrs._run_vitest

    return [
        DefaultInfo(),
        ExternalRunnerTestInfo(
            type = "vitest",
            command = [node_exe, script, node_exe, src_dir],
            run_from_project_root = True,
        )
    ]

vitest_test = rule(
    impl = _vitest_test_impl,
    attrs = {
        "srcs": attrs.list(attrs.source(allow_directory = True), default = []),
        "vitest_config": attrs.option(attrs.source(), default = None),
        "deps": attrs.list(attrs.dep()),
        "_js_runtime": attrs.dep(default = "toolchains//:js"),
        "_run_vitest": attrs.source(default = "//rules/js:run_vitest.mjs"),
    }
)
