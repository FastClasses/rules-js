import { serve } from "bun";

function generateLoremIpsum(words) {
    const lorem = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.".split(" ");
    let result = [];
    for (let i = 0; i < words; i++) {
        result.push(lorem[i % lorem.length]);
    }
    return result.join(" ") + ".";
}

const server = serve({
    port: 3000,
    fetch(req) {
        const url = new URL(req.url);
        if (url.pathname === "/lorem") {
            const words = parseInt(url.searchParams.get("words") || "10", 10);
            return new Response(generateLoremIpsum(words), {
                headers: { "Content-Type": "text/plain" }
            });
        }
        return new Response("Welcome to the Bun Server! Try /lorem?words=25");
    },
});

console.log(`Listening on localhost:${server.port}`);
