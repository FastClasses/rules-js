export function generateLoremIpsum(words: number): string {
    return `Testing Deno, generating ${words} words.`;
}

console.log(generateLoremIpsum(5));
