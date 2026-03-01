load(":providers.bzl", "JsLibraryInfo", "NodeToolchainInfo", "BunToolchainInfo", "DenoToolchainInfo")
load(":node_modules.bzl", "create_node_modules_tree")

def _js_run_impl(ctx):
    npm_deps = [d for d in ctx.attrs.deps if JsLibraryInfo in d]
    node_modules = create_node_modules_tree(ctx, npm_deps)

    copy_map = {}
    for src in ctx.attrs.srcs:
        copy_map[src.short_path] = src
    copy_map["node_modules"] = node_modules

    src_dir = ctx.actions.copied_dir("src_dir", copy_map)

    if ctx.attrs.runtime == "bun":
        exe = ctx.attrs._bun[BunToolchainInfo].bun_exe
    elif ctx.attrs.runtime == "deno":
        exe = ctx.attrs._deno[DenoToolchainInfo].deno_exe
    else:
        exe = ctx.attrs._node[NodeToolchainInfo].node_exe

    script = ctx.attrs._run_native_test
    entry = ctx.attrs.entry

    command = [exe]
    if ctx.attrs.runtime == "deno":
        command.extend(["run", "-A"])
    
    command.extend([script, exe, src_dir, entry] + ctx.attrs.run_args)

    return [
        DefaultInfo(),
        RunInfo(args = command),
    ]

js_run = rule(
    impl = _js_run_impl,
    attrs = {
        "entry": attrs.string(),
        "runtime": attrs.enum(["node", "bun", "deno"], default = "node"),
        "run_args": attrs.list(attrs.string(), default = []),
        "srcs": attrs.list(attrs.source(allow_directory = True), default = []),
        "deps": attrs.list(attrs.dep(), default = []),
        "_node": attrs.dep(default = "toolchains//:node_info"),
        "_bun": attrs.dep(default = "toolchains//:bun_info"),
        "_deno": attrs.dep(default = "toolchains//:deno_info"),
        "_run_native_test": attrs.source(default = "//rules/js:run_native_test.mjs"),
    }
)
