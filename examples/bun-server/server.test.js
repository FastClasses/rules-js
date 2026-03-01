import { serve } from "bun";

function generateLoremIpsum(words) {
    return `Testing Bun, generating ${words} words.`;
}

console.log(generateLoremIpsum(5));
