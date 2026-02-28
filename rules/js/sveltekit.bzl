load(":vite.bzl", "vite_build")

def sveltekit_build(
    name,
    src = "src",
    static = "static",
    config = "svelte.config.js",
    vite_config = "vite.config.ts",
    package_json = "package.json",
    deps = [],
    **kwargs
):
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
