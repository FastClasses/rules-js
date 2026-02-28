load(":providers.bzl", "NodeToolchainInfo")

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
