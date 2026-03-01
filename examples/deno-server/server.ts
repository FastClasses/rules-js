Deno.serve({
    port: 3001,
    defaultHandler: () => new Response("Hello from Deno Server!"),
    handler: (req) => {
        const url = new URL(req.url);
        if (url.pathname === "/lorem") {
            const words = parseInt(url.searchParams.get("words") || "10", 10);
            const lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.".split(" ");
            let result = [];
            for (let i = 0; i < words; i++) {
                result.push(lorem[i % lorem.length]);
            }
            return new Response(result.join(" ") + ".");
        }
        return new Response("Welcome to the Deno Server! Try /lorem?words=25");
    }
});

console.log("Listening on http://localhost:3001/");
