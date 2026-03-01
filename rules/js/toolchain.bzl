load(":providers.bzl", "NodeToolchainInfo", "BunToolchainInfo", "DenoToolchainInfo")

_NODE_SHASUM256 = {
    "node-v22.14.0-linux-x64.tar.gz": "9d942932535988091034dc94cc5f42b6dc8784d6366df3a36c4c9ccb3996f0c2",
    "node-v22.14.0-linux-arm64.tar.gz": "8cf30ff7250f9463b53c18f89c6c606dfda70378215b2c905d0a9a8b08bd45e0",
    "node-v22.14.0-darwin-arm64.tar.gz": "e9404633bc02a5162c5c573b1e2490f5fb44648345d64a958b17e325729a5e42",
    "node-v22.14.0-darwin-x64.tar.gz": "6698587713ab565a94a360e091df9f6d91c8fadda6d00f0cf6526e9b40bed250",
    "node-v22.14.0-win-x64.zip": "55b639295920b219bb2acbcfa00f90393a2789095b7323f79475c9f34795f217",
}

def _get_node_sha256(version, tarball_name):
    key = tarball_name
    if key not in _NODE_SHASUM256:
        fail("No SHA256 known for {}".format(key) +
             "\nPlease add it from https://nodejs.org/dist/v{}/SHASUMS256.txt".format(version))
    return _NODE_SHASUM256[key]

def system_node_toolchain(
    name = "node_toolchain",
    version = "22.14.0",
):
    platforms = {
        "@prelude//os:linux":   ("linux-x64", "tar.gz"),
        "@prelude//os:macos":   ("darwin-arm64", "tar.gz"),
        "@prelude//os:windows": ("win-x64", "zip"),
    }

    urls_map = {}
    sha256_map = {}
    strip_prefix_map = {}
    type_map = {}

    for constraint, (plat, ext) in platforms.items():
        tarball = "node-v{}-{}.{}".format(version, plat, ext)
        url = "https://nodejs.org/dist/v{}/{}".format(version, tarball)
        sha = _get_node_sha256(version, tarball)
        
        urls_map[constraint] = [url]
        sha256_map[constraint] = sha
        strip_prefix_map[constraint] = "node-v{}-{}".format(version, plat)
        type_map[constraint] = ext

    native.http_archive(
        name = name,
        urls = select(urls_map),
        sha256 = select(sha256_map),
        type = select(type_map),
        strip_prefix = select(strip_prefix_map),
        visibility = ["PUBLIC"],
    )

    node_exe_path = select({
        "@prelude//os:linux": "bin/node",
        "@prelude//os:macos": "bin/node",
        "@prelude//os:windows": "node.exe",
    })

    _node_toolchain_rule(
        name = name + "_info",
        node_archive = ":" + name,
        node_exe_path = node_exe_path,
        visibility = ["PUBLIC"],
    )

def _node_toolchain_rule_impl(ctx):
    node_exe = cmd_args(
        ctx.attrs.node_archive[DefaultInfo].default_outputs[0],
        format = "{{}}/{}".format(ctx.attrs.node_exe_path),
    )
    return [
        DefaultInfo(default_output = ctx.attrs.node_archive[DefaultInfo].default_outputs[0]),
        NodeToolchainInfo(node_exe = node_exe),
    ]

_node_toolchain_rule = rule(
    impl = _node_toolchain_rule_impl,
    attrs = {
        "node_archive": attrs.dep(),
        "node_exe_path": attrs.string(),
    },
)


_BUN_SHASUM256 = {
    "bun-v1.1.27-linux-x64.zip": "22bd04407f9b9c73f03936a4acefd943aa7278e5af86ee5b2b98fe60b37c3327",
    "bun-v1.1.27-linux-aarch64.zip": "2ef223096c7b7ddd0438b118f6acb6e924b9ff9ced02f10f29dbfc9941be4c20",
    "bun-v1.1.27-darwin-x64.zip": "2ce2750c1006cb72351aae38bee3a859529a83d19c1105e1238652631d1b741e",
    "bun-v1.1.27-darwin-aarch64.zip": "23f6b160e5d72d4e55fbcda37cdc2c9a18f018e3242be7b94b1ee92aa425bc11",
    "bun-v1.1.27-windows-x64.zip": "606e586b67ebe54eed754bc5253fb59ef3fc90a2fb00e1174a9d49ca5c9c68be",
}

def _get_bun_sha256(version, tarball_name):
    key = "bun-v{}-{}".format(version, tarball_name.replace("bun-", ""))
    if key not in _BUN_SHASUM256:
        fail("No SHA256 known for {}\n".format(key) +
             "Please add it from https://github.com/oven-sh/bun/releases/download/bun-v{}/SHASUMS256.txt".format(version))
    return _BUN_SHASUM256[key]

def system_bun_toolchain(
    name = "bun_toolchain",
    version = "1.1.27",
):
    platforms = {
        "@prelude//os:linux":   "bun-linux-x64.zip",
        "@prelude//os:macos":   "bun-darwin-aarch64.zip",
        "@prelude//os:windows": "bun-windows-x64.zip",
    }

    urls_map = {}
    sha256_map = {}
    for constraint, tarball in platforms.items():
        url = "https://github.com/oven-sh/bun/releases/download/bun-v{}/{}".format(version, tarball)
        urls_map[constraint] = [url]
        sha256_map[constraint] = _get_bun_sha256(version, tarball)

    native.http_archive(
        name = name,
        urls = select(urls_map),
        sha256 = select(sha256_map),
        strip_prefix = select({
            "@prelude//os:linux": "bun-linux-x64",
            "@prelude//os:macos": "bun-darwin-aarch64",
            "@prelude//os:windows": "bun-windows-x64",
        }),
        visibility = ["PUBLIC"],
    )

    bun_exe_path = select({
        "@prelude//os:windows": "bun.exe",
        "DEFAULT": "bun",
    })

    _bun_toolchain_rule(
        name = name + "_info",
        bun_archive = ":" + name,
        bun_exe_path = bun_exe_path,
        visibility = ["PUBLIC"],
    )

def _bun_toolchain_rule_impl(ctx):
    bun_exe = cmd_args(
        ctx.attrs.bun_archive[DefaultInfo].default_outputs[0],
        format = "{{}}/{}".format(ctx.attrs.bun_exe_path),
    )
    return [
        DefaultInfo(default_output = ctx.attrs.bun_archive[DefaultInfo].default_outputs[0]),
        BunToolchainInfo(bun_exe = bun_exe),
    ]

_bun_toolchain_rule = rule(
    impl = _bun_toolchain_rule_impl,
    attrs = {
        "bun_archive": attrs.dep(),
        "bun_exe_path": attrs.string(),
    },
)

_DENO_SHASUM256 = {
    "deno-v1.46.3-x86_64-unknown-linux-gnu.zip": "39bb1d21ad19c16fcb14f9d58fb542c2bccf0cd92c19aee8127ac5112b48bf83",
    "deno-v1.46.3-aarch64-unknown-linux-gnu.zip": "acf7e0110e186fc515a1b7367d9c56a9f0205ad448c1c08ab769b8e3ce6f700f",
    "deno-v1.46.3-x86_64-pc-windows-msvc.zip": "d9428daa1b3763bdf562054d0fc40832658515b7071c7f7e98d61961adc2d61a",
    "deno-v1.46.3-aarch64-apple-darwin.zip": "e74f8ddd6d8205654905a4e42b5a605ab110722a7898aef68bc35d6e704c2946",
}

def _get_deno_sha256(version, tarball_name):
    key = "deno-v{}-{}".format(version, tarball_name.replace("deno-", ""))
    if key not in _DENO_SHASUM256:
        fail("No SHA256 known for {}\n".format(key) +
             "Please add it from the release assets at https://github.com/denoland/deno/releases/download/v{}/".format(version))
    return _DENO_SHASUM256[key]

def system_deno_toolchain(
    name = "deno_toolchain",
    version = "1.46.3",
):
    platforms = {
        "@prelude//os:linux":   "deno-x86_64-unknown-linux-gnu.zip",
        "@prelude//os:macos":   "deno-aarch64-apple-darwin.zip",
        "@prelude//os:windows": "deno-x86_64-pc-windows-msvc.zip",
    }

    urls_map = {}
    sha256_map = {}
    for constraint, tarball in platforms.items():
        url = "https://github.com/denoland/deno/releases/download/v{}/{}".format(version, tarball)
        urls_map[constraint] = [url]
        sha256_map[constraint] = _get_deno_sha256(version, tarball)

    native.http_archive(
        name = name,
        urls = select(urls_map),
        sha256 = select(sha256_map),
        visibility = ["PUBLIC"],
    )

    deno_exe_path = select({
        "@prelude//os:windows": "deno.exe",
        "DEFAULT": "deno",
    })

    _deno_toolchain_rule(
        name = name + "_info",
        deno_archive = ":" + name,
        deno_exe_path = deno_exe_path,
        visibility = ["PUBLIC"],
    )

def _deno_toolchain_rule_impl(ctx):
    deno_exe = cmd_args(
        ctx.attrs.deno_archive[DefaultInfo].default_outputs[0],
        format = "{{}}/{}".format(ctx.attrs.deno_exe_path),
    )
    return [
        DefaultInfo(default_output = ctx.attrs.deno_archive[DefaultInfo].default_outputs[0]),
        DenoToolchainInfo(deno_exe = deno_exe),
    ]

_deno_toolchain_rule = rule(
    impl = _deno_toolchain_rule_impl,
    attrs = {
        "deno_archive": attrs.dep(),
        "deno_exe_path": attrs.string(),
    },
)
