JsLibraryInfo = provider(fields = {
    "package_name": provider_field(typing.Any, default = None),
    "version": provider_field(typing.Any, default = None),
    "out_dir": provider_field(typing.Any, default = None),
    "deps": provider_field(typing.Any, default = None),
})
