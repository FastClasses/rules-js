JsLibraryInfo = provider(fields = {
    "package_name": provider_field(str, default = ""),
    "version": provider_field(str, default = ""),
    "out_dir": provider_field(typing.Any, default = None),
    "deps": provider_field(list, default = []),
})

NodeToolchainInfo = provider(fields = {
    "node_exe": provider_field(typing.Any, default = None),
})

BunToolchainInfo = provider(fields = {
    "bun_exe": provider_field(typing.Any, default = None),
})

DenoToolchainInfo = provider(fields = {
    "deno_exe": provider_field(typing.Any, default = None),
})
