load(":js_run.bzl", "js_run")
load(":svelte_check.bzl", "svelte_check_test")
load(":vite.bzl", "vite_build")

def sveltekit_build(
        name,
        src = "src",
        static = "static",
        config = "svelte.config.js",
        vite_config = "vite.config.ts",
        package_json = "package.json",
        deps = [],
        **kwargs):
    srcs = [
        src,
        config,
        package_json,
    ]

    if static:
        srcs.append(static)

    vite_build(
        name = name,
        srcs = srcs,
        vite_config = vite_config,
        deps = deps,
        out_dir = "build",
        **kwargs
    )

    js_run(
        name = name + "_dev",
        srcs = srcs + [vite_config],
        _run_native_test = "//rules/js:run_vite_dev.mjs",
        entry = "not_used_by_vite_dev",
        run_args = [],
        deps = deps,
    )

    svelte_check_test(
        name = name + "_check",
        srcs = srcs + [
            "tsconfig.json",
        ],
        deps = deps,
    )
