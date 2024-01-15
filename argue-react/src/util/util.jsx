export const isDarkMode = () => window.matchMedia('(prefers-color-scheme: dark)').matches;

export const css_val = (v) => getComputedStyle(document.documentElement).getPropertyValue(v);

export const distance = (source, target) => Math.sqrt(Math.pow(source.x - target.x, 2) + Math.pow(source.y - target.y, 2));
