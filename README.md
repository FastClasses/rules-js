# rules-js for Buck2

It's basically a collection of rules for JavaScript projects to work nicely with Buck2. It handles Node, Bun, and Deno environments without you needing to install them system-wide—it just downloads what it needs and hooks it up for you.

## SvelteKit Stuff

If you're using SvelteKit, the `sveltekit_build` macro is super handy. It automatically spins up a few helpful extra targets based on whatever `name` you give it (let's say you named your target `my_app` in the BUCK file):

- **Build (`//path/to:my_app`)**: The main target. Runs your standard Vite build and spits out the compiled assets.
- **Dev Server (`//path/to:my_app_dev`)**: A target that runs `vite dev`. Just use `buck2 run //path/to:my_app_dev` and you get the interactive server with HMR working perfectly.
- **Static Checking (`//path/to:my_app_check`)**: Runs `svelte-kit sync` followed by `svelte-check` to validate all your types. Run it with `buck2 test //path/to:my_app_check`.

## Just Running & Testing Scripts

If you just need to run some JavaScript:

### `js_run`
Use this for things that need to stay alive or need terminal input, like dev servers. Run it with `buck2 run`. It handles piping output and paths for you.

### `js_test`
Use this for automated testing scripts where you just care about the exit code. Run it with `buck2 test`. It works smoothly across Node, Bun, and Deno.

## Linters & Checkers

We also threw in a few rules for common quality checks:

- **`eslint_check`**: A target to run ESLint (even v9 works!). Fails the `buck2 test` if your code has lint errors.
- **`ts_library`**: Runs the exact TypeScript compiler (`tsc --noEmit`) to just type-check your code without emitting anything.
- **`vitest_test`**: Runs your Vitest unit tests inside the Buck2 test executor.

## Node Package Management (npm-herder)

Under the hood, `rules-js` uses our custom Rust binary called `npm-herder` to bridge JS package managers (like pnpm, bun, or deno) with Buck2. It parses your lockfiles to natively pull all those dependencies into a `vendor` tree, completely securely and isolated per-project.

### Setup & Usage

To hook up `npm-herder` in a new JS package within your repository, first download the `npm-herder` DotSlash executable from the latest release:
```bash
curl -LO https://github.com/FastClasses/rules-js/releases/latest/download/npm-herder
chmod +x npm-herder
```

Then initialize it:
```bash
./npm-herder init 
```
This drops a `herder.toml` file in the directory. You can specify things like:
- Which `lockfile` to track (`pnpm-lock.yaml`, `bun.lock`, or `deno.lock`).
- Where your `vendor_dir` is located.
- Whether you want to skip `devDependencies` by setting `production = true`.

Once configured, just run the update command anytime your lockfile changes:
```bash
./npm-herder update
```
It reads your lockfile, forcefully syncs the `vendor` directory (removing stale packages, verifying hashes, and resolving tarballs via parallel streams), and spits out a giant generated `vendor/BUCK` file containing all the exact Starlark targets (like `//vendor:eslint_9.0.0`) you can import in your own app's `BUCK` file!
