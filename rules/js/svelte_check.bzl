load(":providers.bzl", "JsLibraryInfo", "NodeToolchainInfo")
load(":node_modules.bzl", "create_node_modules_tree")

def _svelte_check_impl(ctx):
    npm_deps = [d for d in ctx.attrs.deps if JsLibraryInfo in d]
    node_modules = create_node_modules_tree(ctx, npm_deps)

    copy_map = {}
    
    for src in ctx.attrs.srcs:
        copy_map[src.short_path] = src

    copy_map["node_modules"] = node_modules

    src_dir = ctx.actions.copied_dir("src_dir", copy_map)

    node_exe = ctx.attrs._node[NodeToolchainInfo].node_exe
    script = ctx.attrs._run_svelte_check

    return [
        DefaultInfo(),
        ExternalRunnerTestInfo(
            type = "svelte_check",
            command = [node_exe, script, node_exe, src_dir],
            run_from_project_root = True,
        )
    ]

svelte_check_test = rule(
    impl = _svelte_check_impl,
    attrs = {
        "srcs": attrs.list(attrs.source(allow_directory = True), default = []),
        "deps": attrs.list(attrs.dep()),
        "_node": attrs.dep(default = "toolchains//:node_info"),
        "_run_svelte_check": attrs.source(default = "//rules/js:run_svelte_check.mjs"),
    }
)
