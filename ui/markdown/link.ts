const SCHEMES_THAT_OPEN_IN_THE_BROWSER = new Set([
  "http:",
  "https:",
  "mailto:",
]);

function urlOf(link: string): URL | undefined {
  try {
    return new URL(link);
  } catch {
    return undefined;
  }
}

export function linkToOpen(link: string): string | undefined {
  const url = urlOf(link);
  if (url === undefined) {
    return undefined;
  }
  if (!SCHEMES_THAT_OPEN_IN_THE_BROWSER.has(url.protocol)) {
    return undefined;
  }
  return url.href;
}
